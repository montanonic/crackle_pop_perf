## What this is about
I've done CracklePop / FizzBuzz enough times to want to spice it up a lot. Rust gives me more control over performance than I'm used to in 
other languages, and so it felt fun for me to do some exploration around the impact different types of performance optimizations could make.

I haven't ever had a need to make something faster in a professional environment, and don't have much history with benchmarking, so this
exercise gave me a lot of exposure.

## High level overview of this Repo
So my long slog of cracklepop implementations (with lots of copy-pasting because function calls can have performance impact, and I didn't
want to mess with #[inline] directives (yet)) is mostly repeated code with small successive variations described in the function name / comment.

My comments are reflective of how I generally comment on my own projects, and they are more dense than usual here because I needed to
reason out loud about what was going on to feel like I was thoughtfully interpreting the various benchmark results.

In the end, my stack-allocated array was only a tiny bit faster than a vector implementation, so that itself was an interesting experiment.
I feel good about my implementation code for it nonetheless, and note in the documentation the obvious shortcomings of the data structure.

I wrote a small module specific to my RC submission that includes my RC crackle pop, compared against one of my fastest implementations.
My intention with grouping them in a separate module was to make it easier to directly compare in code what I would consider to be
a pretty basic (but still thoughtful) implementation, with one that was the end result of iterative optimization. I felt like my main
file could be a bit much to take in, so I wanted to give a simpler entry point that didn't depend on anything in my main module.

## How could this be extended?
I think I might have misjudged the importance of working directly with raw bytes vs. using String. I was thinking that String might have
some UTF8-checking overhead, but I actually think that it only does that for operations that *could* invalidate the String, (or for conversions into a String/&str), and I don't
believe any ops I'm using would involve that. It would be more idiomatic to be working with Strings here instead of Vec<u8> if possible,
and I'd be curious to see if there'd be a performance hit to doing so. The main place I'd change is my manual UTF8 number encoding function,
where I'd use some String function to convert the raw bytes, and almost certainly through an unsafe method, as I'm positive String would
do UTF8 validation otherwise (which is not needed as my UTF8 encoding tested correct).
