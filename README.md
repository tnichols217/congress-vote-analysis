#  Clustering, Dimensionality Reduction, and Predictive Modeling for CSDS 313/413

Ashley Chen and Trevor Nichols

This repo has a project for exploring dimensionality reduction, clustering, and predictive modelling on high dimensional datasets.

We utilize multiple clustering and dimensionality reduction methods in order to see which are sufficient for the analysis of our data.

## Running the code

### Part A

- Using nix: `nix develop` to use a devshell with the necessary dependencies for this project
- Using cargo: `cargo run ./path/to/dataset`

### Part B

- Using nix: `nix develop` to use a devshell with the necessary dependencies for this project, then quarto to run our `.qmd` file

## Building the code or report

- Using nix: `nix build` to build the analysis tool, or use `nix build .#report` to generate the pdf report associated with this project.
