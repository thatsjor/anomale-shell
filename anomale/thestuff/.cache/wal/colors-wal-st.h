const char *colorname[] = {

  /* 8 normal colors */
  [0] = "#0e1c21", /* black   */
  [1] = "#879fa8", /* red     */
  [2] = "#7998a9", /* green   */
  [3] = "#6b91aa", /* yellow  */
  [4] = "#969fa8", /* blue    */
  [5] = "#a5a3a7", /* magenta */
  [6] = "#b1acb1", /* cyan    */
  [7] = "#c2c6c7", /* white   */

  /* 8 bright colors */
  [8]  = "#5d6d72",  /* black   */
  [9]  = "#879fa8",  /* red     */
  [10] = "#7998a9", /* green   */
  [11] = "#6b91aa", /* yellow  */
  [12] = "#969fa8", /* blue    */
  [13] = "#a5a3a7", /* magenta */
  [14] = "#b1acb1", /* cyan    */
  [15] = "#c2c6c7", /* white   */

  /* special colors */
  [256] = "#0e1c21", /* background */
  [257] = "#c2c6c7", /* foreground */
  [258] = "#c2c6c7",     /* cursor */
};

/* Default colors (colorname index)
 * foreground, background, cursor */
 unsigned int defaultbg = 0;
 unsigned int defaultfg = 257;
 unsigned int defaultcs = 258;
 unsigned int defaultrcs= 258;
