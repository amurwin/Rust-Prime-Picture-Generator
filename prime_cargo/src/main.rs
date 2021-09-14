/*
Author: Andrew Murwin - ASUID: 1216207577
Date: May 1, 2020
Description: Honors Contract for CSE240

Summary: From scratch, the program takes command line arguments and outputs a
.bmp file that displays all prime and composite numbers within a range base on 
the color of the pixels. The program uses pre-existing crates to deal with 
command line arguments, multi-threading and benchmarking, but the prime number
checking and bitmap file creation were all written from scratch using only
functions native to Rust.

*/

//crates included for multithreading, command line arguments, and benchmarking
#[macro_use]
extern crate structopt_derive;
extern crate structopt;

//Scopes
use rayon::prelude::*;    //Multithreading
use math::round;          //Rounding for square root
use structopt::StructOpt; //Command line arguments
use std::error::Error;    //File I/O
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::iter::Iterator;  //Step_by function

const BYTES_PER_PIXEL: u8 = 3; // red, green, blue
const FILE_HEADER_SIZE: u8 = 14; //Number of bytes in the file header - predetermined by .bmp format
const INFO_HEADER_SIZE: u8 = 40; //Number of bytes in the info header - predetermined by .bmp format

//A struct designed to store an rgb color
struct Color {
    red: u8,
    green: u8,
    blue: u8
}

///Generates a command line parser based on the contents of the "Opt" struct
#[derive(StructOpt, Debug)]
#[structopt(name = "main")]
struct Opt {

    /// An argument that takes a string for the output file name
    #[structopt(help = "output BMP file name", default_value = "output.bmp",)]
    filename: String,

    /// An argument that takes an unsigned int for the width of the BMP, with a default value of 100
    #[structopt(help = "Width of the BMP file", default_value = "100", multiple = false)]
    width: u32,

    /// An argument that takes an int for the height of the BMP, with a default value of 100
    #[structopt(help = "Height of the BMP file", default_value = "100", multiple = false)]
    height: u32,

    /// An argument that takes an 8-bit unsigned int for the composite red, with a default value of 0
    #[structopt(help = "Value for the red composite (0-255)", default_value = "0", multiple = false)]
    composite_red: u8,

    /// An argument that takes an 8-bit unsigned int for the composite green, with a default value of 0
    #[structopt(help = "Value for the green composite (0-255)", default_value = "0", multiple = false)]
    composite_green: u8,

    /// An argument that takes an 8-bit unsigned int for the composite blue, with a default value of 0
    #[structopt(help = "Value for the blue composite (0-255)", default_value = "0", multiple = false)]
    composite_blue: u8,

    /// An argument that takes an 8-bit unsigned int for the prime red, with a default value of 255
    #[structopt(help = "Value for the red prime (0-255)", default_value = "255", multiple = false)]
    prime_red: u8,

    /// An argument that takes an 8-bit unsigned int for the prime green, with a default value of 255
    #[structopt(help = "Value for the green prime (0-255)", default_value = "255", multiple = false)]
    prime_green: u8,

    /// An argument that takes an 8-bit unsigned int for the prime blue, with a default value of 255
    #[structopt(help = "Value for the blue prime (0-255)", default_value = "255", multiple = false)]
    prime_blue: u8,

    /// An optional argument that takes an 8-bit unsigned int for the number of threads to run, with a default value of one
    #[structopt(help = "Number of threads to run", default_value = "1")]
    jobs: Option<u8>,

    /// An optional argument that takes an in a number to designate the method of prime number calcualation, with the program defaulting
    /// to 1 if the value isn't 2-4 to account for invalid inputs
    #[structopt(default_value = "1", help = "Which prime number generation method to use. Options are:\n1. Multithreaded complex check
                                            \n2. Multithreaded simple check\n3. Single thread complex check\n4. Single thread simple check")]
    method: Option<u8>
}

//Uses a very rudimentary method of caluclating prime numbers, checking 1,2 and the the modulo of each number to determine if its perfectly divisible
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

//Runs a more complex and efficient method of prime number checking, checking for 1,2, evens, and
//then up to the square root of the number. This method is valid because after reaching the square-root,
//All valid combinations are the inverse of already checked ones, and would have been already marked as
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
    //Finds the square root and rounds up to account for perfect squares such as 4, 9, etc
    //Steps by two since the even numbers have already been checked
    for i in (3..((round::floor((num as f64).sqrt(), 1) as u32) + 1)).step_by(2)
    {
        if num % i == 0
        {
            return false;
        };
    }
    return true;
}

//Single-thread method for returning a Vector that represents the numbers in the form of true or false
//Runs efficient prime check if methods is 3, runs normal prime check if method is 4, other values can't happen
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

//Runs the designated prime number calculation, returning a Vector of 1 and 0, representing true and false
//Returns in this matter because the parallel processing self modifies, meaning the values had to be u32s
//This is accounted for later when transforming it into an image
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

//Generates bitmap file header
fn create_bitmap_file_header(height: u32, width: u32, padding_size: u32) -> Vec<u8> {
    let file_size: u32 = (FILE_HEADER_SIZE as u32) + (INFO_HEADER_SIZE as u32) + ((((BYTES_PER_PIXEL as u32)*width) + padding_size) * height);
    
    let mut file_header: Vec<u8> = vec![0; FILE_HEADER_SIZE as usize];
    let file_size_bytes = file_size.to_le_bytes();
    file_header[0] = 66;                                    //'B' - Indicates Windows rather than OS/2
    file_header[1] = 77;                                    //'M' - indicates Windows rather than OS/2
    file_header[2] = file_size_bytes[0];                    //Image file size split into four bites
    file_header[3] = file_size_bytes[1];                    //The bites are writte in little endian style,
    file_header[4] = file_size_bytes[2];                    //Starting with the "smallest" (right-most) byte
    file_header[5] = file_size_bytes[3];                    //Byte 4
    file_header[10] = FILE_HEADER_SIZE + INFO_HEADER_SIZE;  //Location of the start of the actual picture
    return file_header;
}

//Generates bitmap info header
fn create_bitmap_info_header(height: u32, width: u32) -> Vec<u8>
{
    let mut info_header: Vec<u8> = vec![0; INFO_HEADER_SIZE as usize];
    let height_bytes = height.to_le_bytes();
    let width_bytes = width.to_le_bytes();
    info_header[0] = INFO_HEADER_SIZE;          //Size of the info header
    info_header[4] = width_bytes[0];            //Image width in little endian style
    info_header[5] = width_bytes[1];            //Byte 2
    info_header[6] = width_bytes[2];            //Byte 3
    info_header[7] = width_bytes[3];            //Byte 4
    info_header[8] = height_bytes[0];           //Image height in little endian style
    info_header[9] = height_bytes[1];           //Byte 2
    info_header[10] = height_bytes[2];          //Byte 3
    info_header[11] = height_bytes[3];          //Byte 4
    info_header[12] = 1;                        //Number of color planes
    info_header[14] = BYTES_PER_PIXEL*8;        //Bits per pixel
    return info_header;
}

//Generates the bitmap image and writes it to a file
fn generate_bitmap_image(image: Vec<Color>, height: u32, width: u32, file_name: &str)
{
    let padding_size: u32 = (4 - ((width*(BYTES_PER_PIXEL as u32)) % 4)) % 4; //Padding size, rounds out to the 4th byte
    let file_header: Vec<u8> = create_bitmap_file_header(height, width, padding_size);
    let info_header: Vec<u8> = create_bitmap_info_header(height, width);
    
    //Make sure output file is a .bmp by appendiong ".bmp" if it isn't
    let mut name = file_name.to_string();
    if name.len() > 4
    {
        let slice = &name[(name.len()-4)..name.len()];
        if !(slice == ".bmp")
        {
            name.push_str(".bmp");
        }
    }
    else
    {
        name.push_str(".bmp");
    }

    //Creates the output file in write-only mode
    let path = Path::new(&name);
    let display = path.display();
    #[allow(deprecated)]
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why.description()),
        Ok(file) => file,
    };

    //Write the file header and the info header to the file
    for i in file_header
    {
        let _bytes_written = file.write(&[i]);
    }
    for i in info_header
    {
        let _bytes_written = file.write(&[i]);
    }

    //Iterates through every color in the vector and writes the RGB to the file
    //However, the row order is reversed because the origin of a bitmap is in 
    //the bottom left, meaning the file has to be written from the bottom up
    for i in (0..height).rev(){
        for j in 0..width{
            //Colors are printed in reverse order because BMP stores in blue, green, red order
            let _bytes_written = file.write(&[image[((i * width) + j) as usize].blue]);
            let _bytes_written = file.write(&[image[((i * width) + j) as usize].green]);
            let _bytes_written = file.write(&[image[((i * width) + j) as usize].red]);
        }
        //Adding padding bytes to fix alignment
        for _j in 0..padding_size{
            let _bytes_written = file.write(&[0]);
        }
    }
    
}

fn main() {
    //Gets the command line arguments
    let args = Opt::from_args();

    //Sets the multi-thread count according to the number specified in the command line argument
    rayon::ThreadPoolBuilder::new().num_threads(args.jobs.unwrap() as usize).build_global().unwrap();
    
    //Color for the composite numbers
    let composite_color = Color {
        red: args.composite_red,
        green: args.composite_green,
        blue: args.composite_blue
    };
    //Color for the prime numbers
    let prime_color = Color {
        red: args.prime_red,
        green: args.prime_green,
        blue: args.prime_blue
    };
    //Create image and number of iterations
    let mut image = Vec::new();
    let num: u32 = args.width * args.height;

    //Calculate if the option was single threading
    if args.method.unwrap() == 3 || args.method.unwrap() == 4
    {
        let tf_array: Vec<bool> = single_thread_prime(num, args.method.unwrap());
        for n in 0..num //For each number, if true add a prime pixel to the Vector, otherwise add a composite pixel
        {
        if tf_array[n as usize]
            {
                image.push(Color{red: prime_color.red, 
                                green: prime_color.green, 
                                blue: prime_color.blue});
            }
            else
            {
                image.push(Color{red: composite_color.red, 
                                green: composite_color.green, 
                                blue: composite_color.blue});
            }
        }
    }
    //Calculate if the option was multi-threading
    else
    {
        //This covers the default option or if an invalid number is entered
        let mut multi_tf_array: Vec<u32> = multi_thread_prime(num, 1);
        //However, overwrites if option 2 is chosen
        if args.method.unwrap() == 2
        {
            multi_tf_array = multi_thread_prime(num, 2);
        }
        for n in 0..num //For each number, if 1 add a prime pixel to the Vector, otherwise add a composite pixel
        {               //This is where the fact that multithreading self-modifies the array is accounted for
            if multi_tf_array[n as usize] == 1
            {
                image.push(Color{red: prime_color.red, 
                                green: prime_color.green, 
                                blue: prime_color.blue});
            }
            else
            {
                image.push(Color{red: composite_color.red, 
                                green: composite_color.green, 
                                blue: composite_color.blue});
            }
        }
    }
    //Creates the bitmap file
    generate_bitmap_image(image, args.height, args.width, &args.filename); 
}