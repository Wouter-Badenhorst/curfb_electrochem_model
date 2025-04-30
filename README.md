# curfb_electrochem_model

Calculation time with default settings 478s, suspicion file access when running the model slowing calculation time read file data in main and feed to the model.

Calculation time with default settings 43s, after only reading data once in main fail and copying the vector for each model instance, ram usage significantly increased however.

Calculation time with default settings 6s, after removing DIY multithreading code and replacing with Rayon implementation: Memory usage significantly decreased and CPU utilization peaks at 100%


30/05/2025

Significant updates to Rayon for multithreading, additional of plotting software, added data preparation script. Improved retention of elite genes, implemented gradual decrease in mutation intensity, reduced file access occurences, removed usage of json file for settings
Added new parameters for estimations C0c and C2a to better account for data that does not start from ideal. Lower population size and more generations used in current implementation. 

Implemented new code to use multiple cells and account for the extra diffusion with more membrane surface area.
Critically also implemented more dynamic C1 flow from high cocentration to low concentration

Model does not account for flowrate and cell - tank concentration variations