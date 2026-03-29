/* Taken from https://github.com/djpohly/dwl/issues/466 */
#define COLOR(hex)    { ((hex >> 24) & 0xFF) / 255.0f, \
                        ((hex >> 16) & 0xFF) / 255.0f, \
                        ((hex >> 8) & 0xFF) / 255.0f, \
                        (hex & 0xFF) / 255.0f }

static const float rootcolor[]             = COLOR(0x0e1c21ff);
static uint32_t colors[][3]                = {
	/*               fg          bg          border    */
	[SchemeNorm] = { 0xc2c6c7ff, 0x0e1c21ff, 0x5d6d72ff },
	[SchemeSel]  = { 0xc2c6c7ff, 0x7998a9ff, 0x879fa8ff },
	[SchemeUrg]  = { 0xc2c6c7ff, 0x879fa8ff, 0x7998a9ff },
};
