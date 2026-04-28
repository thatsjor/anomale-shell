const char *colorname[] = {

  /* 8 normal colors */
  [0] = "#20200e", /* black   */
  [1] = "#a5a87c", /* red     */
  [2] = "#a4a98a", /* green   */
  [3] = "#a5a996", /* yellow  */
  [4] = "#a5a9a0", /* blue    */
  [5] = "#a7a9a8", /* magenta */
  [6] = "#a2a2aa", /* cyan    */
  [7] = "#c7c7c2", /* white   */

  /* 8 bright colors */
  [8]  = "#72725d",  /* black   */
  [9]  = "#a5a87c",  /* red     */
  [10] = "#a4a98a", /* green   */
  [11] = "#a5a996", /* yellow  */
  [12] = "#a5a9a0", /* blue    */
  [13] = "#a7a9a8", /* magenta */
  [14] = "#a2a2aa", /* cyan    */
  [15] = "#c7c7c2", /* white   */

  /* special colors */
  [256] = "#20200e", /* background */
  [257] = "#c7c7c2", /* foreground */
  [258] = "#c7c7c2",     /* cursor */
};

/* Default colors (colorname index)
 * foreground, background, cursor */
 unsigned int defaultbg = 0;
 unsigned int defaultfg = 257;
 unsigned int defaultcs = 258;
 unsigned int defaultrcs= 258;
