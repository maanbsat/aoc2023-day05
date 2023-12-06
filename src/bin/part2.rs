use std::fs::File;
use std::io::{self, BufRead};
use std::ops::Range;

use anyhow::{Context, Result};
use itertools::Itertools;
use rayon::prelude::*;

const PATH: &str = "input.txt";

struct AlmanacRange {
    source_start: usize,
    destination_start: usize,
    range_length: usize,
}

impl AlmanacRange {
    fn get_destination(&self, source: usize) -> Option<usize> {
        if source < self.source_start || source > self.source_start + self.range_length {
            None
        } else {
            Some(self.destination_start + (source - self.source_start))
        }
    }
}

struct AlmanacRanges {
    ranges: Vec<AlmanacRange>,
}

impl AlmanacRanges {
    fn from_vec(mut ranges: Vec<AlmanacRange>) -> Self {
        ranges.sort_by_key(|r| r.source_start);
        AlmanacRanges { ranges }
    }

    fn get_destination(&self, source: usize) -> usize {
        let res = self
            .ranges
            .binary_search_by_key(&source, |r| r.source_start);
        match res {
            // we found an exact start -> return the destination start
            Ok(r) => self.ranges[r].destination_start,
            // this source is smaller than any range, so we don't have a match
            // -> it's a 1:1 match
            Err(0) => source,
            // this source is after our last range
            Err(i) if i == self.ranges.len() => {
                // first check if it's part of the last range
                match self.ranges.last().unwrap().get_destination(source) {
                    Some(dst) => dst,
                    // it's not part of the last range -> 1:1 match
                    None => source,
                }
            }
            // it's somewhere in the middle: check range before, otherwise 1:1 match
            Err(i) => match self.ranges[i - 1].get_destination(source) {
                Some(dst) => dst,
                None => source,
            },
        }
    }
}

struct Almanac {
    seeds: Vec<Range<usize>>,
    seed_to_soil: AlmanacRanges,
    soil_to_fertilizer: AlmanacRanges,
    fertilizer_to_water: AlmanacRanges,
    water_to_light: AlmanacRanges,
    light_to_temperature: AlmanacRanges,
    temperature_to_humidity: AlmanacRanges,
    humidity_to_location: AlmanacRanges,
}

fn parse_num_sequence(seq: &str) -> Vec<usize> {
    seq.split(' ')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect()
}

fn parse_ranges(seq: &str) -> Vec<Range<usize>> {
    seq.split(' ')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<usize>().unwrap())
        .tuples()
        .map(|(v1, v2)| (v1..(v1 + v2)))
        .collect()
}

fn parse_section(lines: &mut io::Lines<impl BufRead>) -> Result<AlmanacRanges> {
    Ok(AlmanacRanges::from_vec(
        lines
            .map(|l| l.unwrap())
            .take_while(|l| !l.is_empty())
            .filter_map(|l| {
                if l.ends_with(':') {
                    None
                } else {
                    let range = parse_num_sequence(&l);
                    // println!("{:?}", range);
                    assert_eq!(range.len(), 3);
                    Some(AlmanacRange {
                        destination_start: range[0],
                        source_start: range[1],
                        range_length: range[2],
                    })
                }
            })
            .collect(),
    ))
}

fn part2(input: impl BufRead) -> Result<usize> {
    let mut lines = input.lines();
    let l = lines.next().context("can't get first line")??;
    let (header, numbers) = l.split_once(": ").context("can't parse first line")?;
    assert_eq!(header, "seeds");
    let seeds = parse_ranges(numbers);

    lines.next();
    let almanac = Almanac {
        seeds,
        seed_to_soil: parse_section(&mut lines)?,
        soil_to_fertilizer: parse_section(&mut lines)?,
        fertilizer_to_water: parse_section(&mut lines)?,
        water_to_light: parse_section(&mut lines)?,
        light_to_temperature: parse_section(&mut lines)?,
        temperature_to_humidity: parse_section(&mut lines)?,
        humidity_to_location: parse_section(&mut lines)?,
    };

    // now iterate through the seed numbers and map all the way to location
    almanac
        .seeds
        .into_par_iter()
        .map(|r| {
            println!(
                "Iterating through range {} to {}, length {}",
                r.start,
                r.end,
                r.len()
            );
            r
        })
        .flatten()
        .map(|s| almanac.seed_to_soil.get_destination(s))
        .map(|s| almanac.soil_to_fertilizer.get_destination(s))
        .map(|s| almanac.fertilizer_to_water.get_destination(s))
        .map(|s| almanac.water_to_light.get_destination(s))
        .map(|s| almanac.light_to_temperature.get_destination(s))
        .map(|s| almanac.temperature_to_humidity.get_destination(s))
        .map(|s| almanac.humidity_to_location.get_destination(s))
        .min()
        .context("no minimum found")
}

fn main() -> Result<()> {
    let res = part2(io::BufReader::new(File::open(PATH)?))?;
    println!("{res}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t1() -> Result<()> {
        assert_eq!(
            part2(io::BufReader::new(File::open("input_test.txt")?))?,
            46
        );

        Ok(())
    }
}
