# RustOpenStreetMap
My final project for CSCI 4511W Artificial Intelligence @ UMN. The program runs A* throughout Minnesota and uses data provided by OpenStreetMap.


## Example paths
Here is an image of generated paths given two random points and traversed with parallel bi-directional A*.

For 1000 random paths, BDA* takes 3415ms whereas regular 
A* takes 20114ms. This makes bidirectional A* ~5.8x faster.

Out of a sample of 1000 paths mean path length is 221.11 miles and mean node count is 2960.7.

The paths found in BDA* are shown below.


![Paths](.github/bda.png)
