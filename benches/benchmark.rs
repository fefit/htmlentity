use criterion::{criterion_group, criterion_main, Criterion};
use htmlentity::entity::{decode, encode};
fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("encode", |b| {
    b.iter(|| {
      encode(
        r##"
        <div class="s-rank-title">
          <a href="http://top.baidu.com/?fr=mhd_card" target="_blank">
              <div class="title-text c-font-medium c-color-t">
                  热榜
              </div>
          </a>
          <a class="hot-refresh c-font-normal c-color-gray2"> 
              <i class="c-icon">&#xe619;</i><span class="hot-refresh-text">换一换</span>
          </a>
      </div>
        "##
          .as_bytes(),
        &Default::default(),
        &Default::default(),
      );
    })
  });
  c.bench_function("decode", |b| {
    b.iter(|| {
      decode(r##"
        &lt;&#x64;&#x69;&#x76;&#x20;&#x63;&#x6C;&#x61;&#x73;&#x73;&equals;&quot;&#x73;&#x2D;&#x72;&#x61;&#x6E;&#x6B;&#x2D;&#x74;&#x69;&#x74;&#x6C;&#x65;&quot;&gt;&NewLine;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&lt;&#x61;&#x20;&#x68;&#x72;&#x65;&#x66;&equals;&quot;&#x68;&#x74;&#x74;&#x70;&colon;&sol;&sol;&#x77;&#x77;&#x77;&period;&#x61;&#x62;&#x63;&period;&#x63;&#x6F;&#x6D;&sol;&quest;&#x66;&#x72;&equals;&#x6D;&#x68;&#x64;&lowbar;&#x63;&#x61;&#x72;&#x64;&quot;&#x20;&#x74;&#x61;&#x72;&#x67;&#x65;&#x74;&equals;&quot;&lowbar;&#x62;&#x6C;&#x61;&#x6E;&#x6B;&quot;&gt;&NewLine;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&lt;&#x64;&#x69;&#x76;&#x20;&#x63;&#x6C;&#x61;&#x73;&#x73;&equals;&quot;&#x74;&#x69;&#x74;&#x6C;&#x65;&#x2D;&#x74;&#x65;&#x78;&#x74;&#x20;&#x63;&#x2D;&#x66;&#x6F;&#x6E;&#x74;&#x2D;&#x6D;&#x65;&#x64;&#x69;&#x75;&#x6D;&#x20;&#x63;&#x2D;&#x63;&#x6F;&#x6C;&#x6F;&#x72;&#x2D;&#x74;&quot;&gt;&NewLine;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x70ED;&#x699C;&NewLine;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&lt;&sol;&#x64;&#x69;&#x76;&gt;&NewLine;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&lt;&sol;&#x61;&gt;&NewLine;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&lt;&#x61;&#x20;&#x63;&#x6C;&#x61;&#x73;&#x73;&equals;&quot;&#x68;&#x6F;&#x74;&#x2D;&#x72;&#x65;&#x66;&#x72;&#x65;&#x73;&#x68;&#x20;&#x63;&#x2D;&#x66;&#x6F;&#x6E;&#x74;&#x2D;&#x6E;&#x6F;&#x72;&#x6D;&#x61;&#x6C;&#x20;&#x63;&#x2D;&#x63;&#x6F;&#x6C;&#x6F;&#x72;&#x2D;&#x67;&#x72;&#x61;&#x79;&#x32;&quot;&gt;&#x20;&NewLine;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&lt;&#x69;&#x20;&#x63;&#x6C;&#x61;&#x73;&#x73;&equals;&quot;&#x63;&#x2D;&#x69;&#x63;&#x6F;&#x6E;&quot;&gt;&amp;&num;&#x78;&#x65;&#x36;&#x31;&#x39;&semi;&lt;&sol;&#x69;&gt;&lt;&#x73;&#x70;&#x61;&#x6E;&#x20;&#x63;&#x6C;&#x61;&#x73;&#x73;&equals;&quot;&#x68;&#x6F;&#x74;&#x2D;&#x72;&#x65;&#x66;&#x72;&#x65;&#x73;&#x68;&#x2D;&#x74;&#x65;&#x78;&#x74;&quot;&gt;&#x6362;&#x4E00;&#x6362;&lt;&sol;&#x73;&#x70;&#x61;&#x6E;&gt;&NewLine;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&#x20;&lt;&sol;&#x61;&gt;&NewLine;&#x20;&#x20;&#x20;&#x20;&lt;&sol;&#x64;&#x69;&#x76;&gt;
      "##.as_bytes());
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
