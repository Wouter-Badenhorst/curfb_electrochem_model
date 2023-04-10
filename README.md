# curfb_electrochem_model



Calculation time with default settings 478s, suspicion file access when running the model slowing calculation time read file data in main and feed to the model.

Calculation time with default settings 43s, after only reading data once in main fail and copying the vector for each model instance, ram usage significantly increased however.
