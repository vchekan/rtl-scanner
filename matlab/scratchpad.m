load("./data/dump.mat");

n = (raw_bytes - 127)/127.0;
c=n(:, 1:2:end) + n(:, 2:2:end)*i;
avg=sum(c')/size(c)(2);
ca=c-avg';
f=fft(c);
# calc fft across 2nd dimention (rows)
fa=fft(ca, [], 2);
p=periodogram(fa(1,:));

save "./data/n.mat" n
save "./data/c.mat" c
save "./data/avg.mat" avg
save "./data/ca.mat" ca
save "./data/f.mat" f
save "./data/fa.mat" fa
save "./data/p.mat" p




