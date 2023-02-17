mod extra;
use std::collections::HashMap;

pub use extra::parse_extra;

mod pec;
pub use pec::parse_pec;

mod pgr;
pub use pgr::parse_phigros;

mod rpe;
pub use rpe::{parse_rpe, RPE_HEIGHT, RPE_WIDTH};

fn process_lines(v: &mut [crate::core::JudgeLine]) {
    use crate::ext::NotNanExt;
    let mut times = Vec::new();
    // TODO optimize using k-merge sort
    let sorts = v
        .iter()
        .map(|line| {
            let mut idx: Vec<usize> = (0..line.notes.len()).collect();
            idx.sort_by_key(|id| line.notes[*id].time.not_nan());
            idx
        })
        .collect::<Vec<_>>();
    for (line, idx) in v.iter_mut().zip(sorts.iter()) {
        let v = &mut line.notes;
        let mut i = 0;
        while i < v.len() {
            times.push((v[idx[i]].time.not_nan(), v[idx[i]].fake));
            let mut j = i + 1;
            while j < v.len() && v[idx[j]].time == v[idx[i]].time {
                j += 1;
            }
            if j != i + 1 {
                times.push((v[idx[i]].time.not_nan(), v[idx[i]].fake));
            }
            i = j;
        }
    }
    let mut mt = Vec::new();
    let mut count = HashMap::new();
    if !times.is_empty() {
        for i in 0..times.len() {
            let (time, fake) = times[i];
            let entry = count.entry(time).or_insert((0, 0));
            entry.0 += 1;
            if !fake {
                entry.1 += 1;
            }
        }
    
        for (time, (n, m)) in count {
            if n >= 2 {
                let real = m >= 2;
                mt.push((time, real));
            }
        }
    }
    mt.sort_by(|a, b| a.0.cmp(&b.0));
    for (line, idx) in v.iter_mut().zip(sorts.iter()) {
        let mut i = 0;
        for id in idx {
            let note = &mut line.notes[*id];
            let time = note.time;
            while i < mt.len() && *mt[i].0 < time {
                i += 1;
            }
            let (t, real) = mt[i];
            if i < mt.len() && t == time && (note.fake || real) {
                note.multiple_hint = true;
            }
        }
    }
}

#[rustfmt::skip]
pub const RPE_TWEEN_MAP: [crate::core::TweenId; 30] = {
    use crate::core::{easing_from as e, TweenMajor::*, TweenMinor::*};
    [
        2, 2, // linear
        e(Sine, Out), e(Sine, In),
        e(Quad, Out), e(Quad, In),
        e(Sine, InOut), e(Quad, InOut),
        e(Cubic, Out), e(Cubic, In),
        e(Quart, Out), e(Quart, In),
        e(Cubic, InOut), e(Quart, InOut),
        e(Quint, Out), e(Quint, In),
        e(Expo, Out), e(Expo, In),
        e(Circ, Out), e(Circ, In),
        e(Back, Out), e(Back, In),
        e(Circ, InOut), e(Back, InOut),
        e(Elastic, Out), e(Elastic, In),
        e(Bounce, Out), e(Bounce, In),
        e(Bounce, InOut), e(Elastic, InOut),
    ]
};
