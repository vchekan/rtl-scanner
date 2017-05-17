load("/home/vadim/projects/rtl-scanner-rust/data/raw.mat");

n = (raw_bytes - 127)/127.0;
c=n(:, 1:2:end) + n(:, 2:2:end)*i;
avg=sum(c)/size(c)(2);
ca=c-avg;
f=fft(c);
fa=fft(ca);
