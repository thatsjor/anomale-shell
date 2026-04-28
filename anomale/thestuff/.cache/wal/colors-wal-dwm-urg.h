static const char norm_fg[] = "#c7c7c2";
static const char norm_bg[] = "#20200e";
static const char norm_border[] = "#72725d";

static const char sel_fg[] = "#c7c7c2";
static const char sel_bg[] = "#a4a98a";
static const char sel_border[] = "#c7c7c2";

static const char urg_fg[] = "#c7c7c2";
static const char urg_bg[] = "#a5a87c";
static const char urg_border[] = "#a5a87c";

static const char *colors[][3]      = {
    /*               fg           bg         border                         */
    [SchemeNorm] = { norm_fg,     norm_bg,   norm_border }, // unfocused wins
    [SchemeSel]  = { sel_fg,      sel_bg,    sel_border },  // the focused win
    [SchemeUrg] =  { urg_fg,      urg_bg,    urg_border },
};
