#[macro_use]
extern crate criterion;
extern crate num_cpus;

use criterion::Criterion;
use rayon::prelude::*;
use math::round;

//Uses a very rudimentary method of caluclating prime numbers, checking 1,2 
//and the the modulo of each number to determine if its perfectly divisible
fn basic_is_prime(num: u32) -> bool {
    if num == 1 {
        return false;
    }
    if num == 2 {
        return true;
    }
    for i in 2..(num)
    {
        if num % i == 0
        {
            return false;
        };
    }
    return true;
}

//Single-thread method for returning a Vector that represents the numbers in 
//the form of true or false. Runs efficient prime check if methods is 3, 
//otherwise runs simple prime check
fn single_thread_prime(num: u32, method: u8) -> Vec<bool>
{

    let mut tf_vector = Vec::new();
    if method == 3
    {
        for n in 1..(num + 1)
        {
            if complex_is_prime(n)
            {
                tf_vector.push(true);
            }
            else
            {
                tf_vector.push(false);
            }
        }
    }
    else
    {
        for n in 1..(num + 1)
        {
            if basic_is_prime(n)
            {
                tf_vector.push(true);
            }
            else
            {
                tf_vector.push(false);
            }
        }
    }
    return tf_vector;
} 

//Runs a more complex and efficient method of prime number checking, checking
//for 1,2, evens, and then up to the square root of the number. This method is
//valid because after reaching the square-root, all valid combinations are the
//inverse of already checked ones, and would have been already marked as
//composite if they worked
fn complex_is_prime(num: u32) -> bool {
    if num == 1
    {
        return false;
    }
    if num == 2
    {
        return true;
    }
    if (num > 2) && (num % 2 == 0)
    {
        return false;
    }
    for i in (3..((round::floor((num as f64).sqrt(), 1) as u32) + 1)).step_by(2)
    {
        if num % i == 0
        {
            return false;
        };
    }
    return true;
}

//Runs the designated prime number calculation, returning a Vector of 1 and 0,
//representing true and false. Returns in this matter because the parallel
//processing self modifies, meaning the values had to be u32s. This is accounted 
//for later when transforming it into an image
fn multi_thread_prime(num: u32, method: u8) -> Vec<u32>
{
    let mut num_array: Vec<u32> = (1..(num + 1)).collect();
    if method == 1 {
        num_array.par_iter_mut().for_each(|p| *p = if complex_is_prime(*p) {1} else {0});
    }
    else
    {
        num_array.par_iter_mut().for_each(|p| *p = if basic_is_prime(*p) {1} else {0});
    }
    return num_array;
}


//Benchmarks each of the four possible combinations and prints the number of available cores when multithreading
fn criterion_benchmark(c: &mut Criterion) {
    rayon::ThreadPoolBuilder::new().num_threads(10).build_global().unwrap();
    c.bench_function("multi thread complex", |b| b.iter(|| multi_thread_prime(10000, 1)));
    println!("CPUs: {}, Physical Cores: {}\n", num_cpus::get(), num_cpus::get_physical());
    c.bench_function("multi thread simple", |b| b.iter(|| multi_thread_prime(10000, 2)));
    println!("CPUs: {}, Physical Cores: {}\n", num_cpus::get(), num_cpus::get_physical());
    c.bench_function("single thread complex", |b| b.iter(|| single_thread_prime(10000, 3)));
    c.bench_function("single thread simple", |b| b.iter(|| single_thread_prime(10000, 4)));
}

//Runs benchmark
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);