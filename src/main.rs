use linfa::prelude::*;
use linfa_clustering::KMeans;
use linfa_nn::distance::L2Dist;
use linfa_reduction::{Pca, ReductionError};
use ndarray::{Array2, Axis, ShapeError, s};
use plotters::prelude::*;
use statrs::statistics::Statistics;
use strum_macros::{EnumString, VariantNames};
use std::collections::HashMap;
use std::{error::Error, fs, env};
use rand::seq::SliceRandom;
use rand::rng;

#[derive(Debug, PartialEq, EnumString, VariantNames)]
#[strum(serialize_all = "kebab-case")]
enum NamedColumns {
    Party,
}

#[derive(Debug, PartialEq, EnumString, VariantNames)]
#[strum(serialize_all = "kebab-case")]
enum Columns {
    HandicappedInfants,
    WaterProjectCostSharing,
    AdoptionOfTheBudgetResolution,
    PhysicianFeeFreeze,
    ElSalvadorAid,
    ReligiousGroupsInSchools,
    AntiSatelliteTestBan,
    AidToNicaraguanContras,
    MxMissile,
    Immigration,
    SynfuelsCorporationCutback,
    EducationSpending,
    SuperfundRightToSue,
    Crime,
    DutyFreeExports,
    ExportAdministrationActSouthAfrica,
}

#[derive(Debug)]
enum LoadError {
    IOError(Box<dyn Error>),
    ShapeError(ShapeError),
}

fn load_votes(path: &str) -> Result<Array2<f64>, LoadError> {
    let mut rdr = csv::Reader::from_reader(
        fs::File::open(path).map_err(|e| LoadError::IOError(Box::new(e)))?
    );

    let mut arr = Vec::new();

    for result in rdr.records() {
        let record = result.map_err(|e| LoadError::IOError(Box::new(e)))?;

        let row: Vec<f64> = record.iter()
            .map(|item| match item.trim() {
                "y" | "republican" => 1.0,
                "n" | "democrat"   => -1.0,
                _                  => 0.0,
            })
            .collect();

        arr.push(row);
    }

    let rows = arr.len();
    let cols = arr[0].len(); // assumes non-empty & rectangular

    let flat: Vec<f64> = arr.clone().into_iter().flatten().collect();

    let arr = Array2::from_shape_vec((rows, cols), flat).map_err(LoadError::ShapeError)?;

    Ok(arr)
}


#[derive(Debug)]
enum PCAError {
    ReductionError(ReductionError),
}

fn perform_pca(votes: &Array2<f64>) -> Result<Pca<f64>, PCAError> {
    let dataset = DatasetBase::new(votes.clone(), ());
    
    let pca = Pca::params(3)
        .fit(&dataset)
        .map_err(PCAError::ReductionError)?;
    
    Ok(pca)
}

fn transform_pca(pca: &Pca<f64>, votes: &Array2<f64>) -> Array2<f64> {
    let db = DatasetBase::new(votes.clone(), ());
    pca.transform(db).records().to_owned()
}

fn plot_variance<'a>(variance: &Vec<f64>, path: &'a str) -> Result<(), DrawingAreaErrorKind<<SVGBackend<'a> as DrawingBackend>::ErrorType>> {
    let root = SVGBackend::<'a>::new(path, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("Cumulative Variance Explained", ("sans-serif", 30))
        .margin(20)
        .set_label_area_size(LabelAreaPosition::Left, 30)
        .set_label_area_size(LabelAreaPosition::Bottom, 30)
        .build_cartesian_2d(0..variance.len()-1, 0f64..1f64)
        ?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (0..variance.len()).map(|i| (i, variance[i])),
            &RED,
        ))
        ?;

    Ok(())
}

fn scatter_plot(data: &Array2<f64>, parties: &Vec<isize>, filename: &str, x: usize, y: usize) {
    let root = SVGBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let xs: Vec<f64> = data.column(x).to_vec();
    let ys: Vec<f64> = data.column(y).to_vec();

    let xmin = xs.clone().min();
    let xmax = xs.clone().max();
    let ymin = ys.clone().min();
    let ymax = ys.clone().max();

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("PC{} vs PC{}", x+1, y+1), ("sans-serif", 25))
        .margin(20)
        .build_cartesian_2d(xmin..xmax, ymin..ymax)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart.draw_series(
        xs.iter()
            .zip(ys.iter())
            .zip(parties.iter())
            .map(|((&x, &y), p)| {
                let color = match *p {
                    1 => &RED,
                    -1 => &BLUE,
                    _ => &BLACK,
                };
                Circle::new((x, y), 4, color.filled())
            }),
    ).unwrap();
}

#[derive(Debug)]
enum KMeansError {
    LinfaError(linfa_clustering::KMeansError),
}

fn perform_kmeans(
    data: &Array2<f64>,
    k: usize,
) -> Result<KMeans<f64, L2Dist>, KMeansError> {
    let dataset = DatasetBase::new(data.clone(), ());

    let model = KMeans::params(k)
        .max_n_iterations(200)
        .fit(&dataset)
        .map_err(KMeansError::LinfaError)?;

    Ok(model)
}

fn kmeans_labels(
    model: &KMeans<f64, L2Dist>,
    data: &Array2<f64>
) -> Vec<usize> {
    let dataset = DatasetBase::new(data.clone(), ());
    let assigned = model.predict(&dataset);
    assigned.iter().map(|i| i.clone()).collect()
}

fn permute_dataset(data: &Array2<f64>) -> Array2<f64> {
    let mut rng = rng();
    let mut permuted = data.clone();

    for mut row in permuted.axis_iter_mut(Axis(0)) {
        let mut row_vec = row.to_vec();
        row_vec.shuffle(&mut rng);
        row.assign(&Array2::from_shape_vec((1, row_vec.len()), row_vec).unwrap().row(0));
    }

    permuted
}

fn permutation_test(
    original_fit: KMeans<f64, L2Dist>,
    data: &Array2<f64>,
    k: usize,
    n_permutations: usize
) -> f64 {
    let orig_score = original_fit.inertia();

    let mut permuted_scores = Vec::with_capacity(n_permutations);
    for _ in 0..n_permutations {
        let permuted_data = permute_dataset(data);
        let kmeans_perm = perform_kmeans(&permuted_data, k).unwrap();
        let score = kmeans_perm.inertia();
        permuted_scores.push(score);
    }

    let count_better = permuted_scores
        .iter()
        .filter(|&&s| s <= orig_score)
        .count();

    let p_value = (count_better as f64 + 1.0) / (n_permutations as f64 + 1.0);

    println!("Original inertia: {:.4}", orig_score);
    println!("Permutation scores (first 10): {:?}", &permuted_scores[..10.min(permuted_scores.len())]);
    println!("p-value: {:.4}", p_value);

    p_value
}

fn mutual_info(labels: &Vec<isize>, truth: &Vec<isize>) -> f64 {
    let n = labels.len() as f64;

    // Count occurrences
    let mut label_counts = HashMap::new();
    let mut truth_counts = HashMap::new();
    let mut joint_counts = HashMap::new();

    for (&l, &t) in labels.iter().zip(truth.iter()) {
        *label_counts.entry(l).or_insert(0) += 1;
        *truth_counts.entry(t).or_insert(0) += 1;
        *joint_counts.entry((l, t)).or_insert(0) += 1;
    }

    // MI = sum_{l,t} P(l,t) * log(P(l,t)/(P(l)*P(t)))
    let mut mi = 0.0;
    for ((l, t), &c) in &joint_counts {
        let p_lt = c as f64 / n;
        let p_l = label_counts[l] as f64 / n;
        let p_t = truth_counts[t] as f64 / n;
        mi += p_lt * (p_lt / (p_l * p_t)).ln();
    }

    mi
}

fn main() {
    // Get file
    let args: Vec<String> = env::args().collect();
    let votes_path = &args[1];

    // Load File
    let votes = load_votes(votes_path).unwrap();
    let vote_view = votes.slice(s![.., 1..]).to_owned();

    // Create PCA
    let pca = perform_pca(&vote_view).unwrap();
    let transformed = transform_pca(&pca, &vote_view);

    // Plot PCA
    plot_variance(&pca.explained_variance_ratio().to_vec(), "./variance.svg").unwrap();

    let parties = votes.column(0).map(|i| i.clone()).into_iter().map(|i| i as isize).collect();

    scatter_plot(&transformed, &parties, "pc1_pc2.svg", 0, 1);
    scatter_plot(&transformed, &parties, "pc1_pc3.svg", 0, 2);
    scatter_plot(&transformed, &parties, "pc2_pc3.svg", 1, 2);

    // Clustering
    let k = 2; // Democrat / Republican
    let kmeans_pca = perform_kmeans(&transformed, k).unwrap();

    // Cluster on raw votes
    let kmeans_raw = perform_kmeans(&vote_view, k).unwrap();

    // Cluster labels
    let labels_pca: Vec<isize> = kmeans_labels(&kmeans_pca, &transformed).into_iter().map(|i| match i {
        0 => -1,
        1 => 1,
        _ => 0
    }).collect();

    let labels_raw: Vec<isize> = kmeans_labels(&kmeans_raw, &vote_view).into_iter().map(|i| match i {
        0 => -1,
        1 => 1,
        _ => 0
    }).collect();

    // Plot clusters
    scatter_plot(&transformed, &labels_pca, "kmeans_pc1_pc2.svg", 0, 1);
    scatter_plot(&transformed, &labels_raw, "kmeans_raw.svg", 0, 1);

    // Perform tests and generate statistics
    // println!("Permutation Test for PCA:");
    // let p_value_pca = permutation_test(kmeans_pca, &vote_view, 2, 200);
    // println!("Permutation Test for PCA:");
    // let p_value_raw = permutation_test(kmeans_raw, &vote_view, 2, 200);


    let mi = mutual_info(&labels_pca, &parties);
    println!("Mutual information between clusters and parties PCA: {:.4}", mi);
    let mi = mutual_info(&labels_raw, &parties);
    println!("Mutual information between clusters and parties Raw: {:.4}", mi);
}
