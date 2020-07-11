#[allow(dead_code)]
pub enum LabelRange {
  Any,
  Included,
  Excluded,
}

pub fn generate_labels(dmin: f64, dmax: f64, max_labels: f64, label_inclusion: LabelRange) -> Vec<f64> {
  let mut outmin: f64 = 1.0;
  let mut outmax: f64 = 1.0;
  let mut outstep: f64 = 1.0;

  let mut best_score = -2.0;

  'j_loop: for j in 1..=u64::MAX {
    let j = j as f64;

    for (qpos, &q) in Q.iter().enumerate() {
      let sm = simplicity_max(qpos, Q.len(), j);

      if (score(1., sm, 1., 1.)) < best_score {
        break 'j_loop;
      }

      'k_loop: for k in 2..u64::MAX {
        let k = k as f64;
        let dm = density_max(k, max_labels);

        if (score(1., sm, dm, 1.)) < best_score {
          break 'k_loop;
        }

        let delta = (dmax - dmin) / (k + 1.0) / j / q;

        'z_loop: for z in (delta.log10().ceil() as u64)..=u64::MAX {
          let z = z as f64;

          let step = j * q * 10.0f64.powf(z);
          let cm = coverage_max(dmin, dmax, step * (k - 1.0));

          let scrt = score(cm, sm, dm, 1.);
          if scrt < best_score {
            break 'z_loop;
          }

          let min_start = (dmax / step).floor() * j - (k - 1.0) * j;
          let max_start = (dmin / step).ceil() * j;

          if min_start > max_start {
            break 'z_loop;
          }

          let mut start = min_start;
          while start <= max_start {
            let lmin = start * (step / j);
            let lmax = lmin + step * (k - 1.0);
            let lstep = step;

            let c = coverage(dmin, dmax, lmin, lmax);
            let s = simplicity(qpos, Q.len(), j, lmin, lmax, lstep);
            let g = density(k, max_labels, dmin, dmax, lmin, lmax);
            let l = 1.0;

            let score = score(c, s, g, l);

            start += 1.0;
            match label_inclusion {
              _ if score <= best_score => (),
              LabelRange::Any | LabelRange::Included if ((lmin < dmin) && (lmax > dmax)) => {
                continue;
              }
              LabelRange::Any | LabelRange::Excluded if ((lmin > dmin) && (lmax < dmax)) => {
                continue;
              }
              _ => {
                best_score = score;
                outmin = lmin;
                outmax = lmax;
                outstep = lstep;
              }
            }
          }
        }
      }
    }
  }

  let mut result = Vec::<f64>::with_capacity(((outmax - outmin) / outstep) as usize);
  while outmin <= outmax {
    result.push(outmin);
    outmin += outstep;
  }

  result
}

const W: [f64; 4] = [0.2, 0.25, 0.5, 0.05];
const Q: [f64; 6] = [1.0, 5.0, 2.0, 2.5, 4.0, 3.0];

fn floored_mod(a: f64, n: f64) -> f64 {
  a - n * (a / n).floor()
}

fn simplicity(qpos: usize, qlen: usize, j: f64, lmin: f64, lmax: f64, lstep: f64) -> f64 {
  let v = if floored_mod(lmin, lstep) < 1e-10 && lmin <= 0. && lmax >= 0. {
    1.0
  } else {
    0.0
  };

  1.0 - (qpos as f64) / (qlen as f64 - 1.0) + v - j
}

fn simplicity_max(qpos: usize, qlen: usize, j: f64) -> f64 {
  1.0 - (qpos as f64) / (qlen as f64 - 1.0) - j + 1.0
}

fn coverage(dmin: f64, dmax: f64, lmin: f64, lmax: f64) -> f64 {
  1.0 - 0.5 * ((dmax - lmax).powi(2) + (dmin - lmin).powi(2)) / (0.1 * (dmax - dmin)).powi(2)
}

fn coverage_max(dmin: f64, dmax: f64, span: f64) -> f64 {
  let range = dmax - dmin;
  if span > range {
    let half = (span - range) / 2.0;
    1.0 - 0.5 * (half.powi(2) + half.powi(2)) / (0.1 * range).powi(2)
  } else {
    1.0
  }
}

fn density(k: f64, m: f64, dmin: f64, dmax: f64, lmin: f64, lmax: f64) -> f64 {
  let r = (k - 1.0) / (lmax - lmin);
  let rt = (m - 1.0) / (lmax.max(dmax) - lmin.min(dmin));
  2.0 - (r / rt).max(rt / r)
}

fn density_max(k: f64, m: f64) -> f64 {
  if k >= m {
    2.0 - (k - 1.0) / (m - 1.0)
  } else {
    1.0
  }
}

fn score(c: f64, s: f64, g: f64, l: f64) -> f64 {
  W[0] * c + W[1] * s + W[2] * g + W[3] * l
}

#[cfg(test)]
mod test {
  #[test]
  fn test_wilkinson_extended() {
    let labels = super::generate_labels(1.0, 10.0, 5.0, super::LabelRange::Any);
    assert_eq!(labels, vec![0.0, 2.5, 5.0, 7.5, 10.0]);
  }
}
