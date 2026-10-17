# Sources of the night sky

Made by `scripts/bake-sky/bake-sky.py` (run it again to change the magnitude limit, the image size or the figures).

- `stars.bin`: the stars brighter than magnitude 6.5 (8,920) from the **HYG Database v4.4** (David Nash, astronexus.com/hyg),
  which merges the Hipparcos, Yale Bright Star and Gliese catalogues. Licence **CC BY-SA 4.0**
  (creativecommons.org/licenses/by-sa/4.0/): this file is an adaptation (positions, magnitudes and colours kept, the
  rest dropped, packed to 6 bytes a star) and stays under the same licence. The 89 line segments are the project's own
  drawing, `constellations.txt`.
- `milkyway.jpg`: the "Milky Way background" of **NASA/Goddard Space Flight Center Scientific Visualization Studio,
  Deep Star Maps 2020** (svs.gsfc.nasa.gov/4851), built from **Gaia DR2: ESA/Gaia/DPAC**; 4096 x 2048 reduced to
  2048 x 1024, toned for the game. Credit: NASA/Goddard Space Flight Center Scientific Visualization Studio; Gaia DR2:
  ESA/Gaia/DPAC.
