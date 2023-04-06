close all
data = readtable('output.csv');
nexttile
plot(data.Time,data.c1c)
title("c1c")
hold on
plot(data.Time,data.c0c)
title("c0c")
hold on
plot(data.Time,data.c1a)
title("c1a")
hold on
plot(data.Time,data.c2a)
title("c2a")
legend("C1catholyte", "C0catholyte", "C1anolyte", "C2anolyte")

data_1 = readtable('data.csv');
figure(2)
nexttile
plot(data_1.Time,data_1.Voltage)
ylim([0.3, 0.9])
title("Voltage")
xlabel("Minutes (s)")
ylabel("Voltage (V)")
grid on
hold on
plot(data.Time,data.Voltage)
ylim([0.3, 0.9])
title("Voltage")
xlabel("Minutes (s)")
ylabel("Voltage (V)")




clc
