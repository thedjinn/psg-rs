/// The amount of times to oversample/decimate.
pub const DECIMATE_FACTOR: usize = 8;

/// The size of the windowed sinc FIR filter's impulse response.
pub const FIR_SIZE: usize = 192;

/// An 8x downsampler (by decimation) and anti-aliasing windowed sinc FIR filter.
pub struct Decimator {
    pub buffer: [f64; FIR_SIZE * 2]
}

impl Decimator {
    /// Initialize a new decimator.
    pub fn new() -> Self {
        Self {
            buffer: [0.0; FIR_SIZE * 2]
        }
    }

    /// Apply anti-alias filter and downsample.
    #[allow(clippy::excessive_precision)]
    pub fn render(&mut self, start: usize) -> f64 {
        // Help the optimizer a little by eliminating the bounds check. This will make the copy
        // operation at the bottom of the method more efficient.
        assert!(start <= FIR_SIZE);

        let buffer = &mut self.buffer[start..start + FIR_SIZE];

        let result = -0.0000046183113992051936 * (buffer[1]  + buffer[191]) +
            -0.00001117761640887225  * (buffer[2]  + buffer[190]) +
            -0.000018610264502005432 * (buffer[3]  + buffer[189]) +
            -0.000025134586135631012 * (buffer[4]  + buffer[188]) +
            -0.000028494281690666197 * (buffer[5]  + buffer[187]) +
            -0.000026396828793275159 * (buffer[6]  + buffer[186]) +
            -0.000017094212558802156 * (buffer[7]  + buffer[185]) +
             0.000023798193576966866 * (buffer[9]  + buffer[183]) +
             0.000051281160242202183 * (buffer[10] + buffer[182]) +
             0.00007762197826243427  * (buffer[11] + buffer[181]) +
             0.000096759426664120416 * (buffer[12] + buffer[180]) +
             0.00010240229300393402  * (buffer[13] + buffer[179]) +
             0.000089344614218077106 * (buffer[14] + buffer[178]) +
             0.000054875700118949183 * (buffer[15] + buffer[177]) +
            -0.000069839082210680165 * (buffer[17] + buffer[175]) +
            -0.0001447966132360757   * (buffer[18] + buffer[174]) +
            -0.00021158452917708308  * (buffer[19] + buffer[173]) +
            -0.00025535069106550544  * (buffer[20] + buffer[172]) +
            -0.00026228714374322104  * (buffer[21] + buffer[171]) +
            -0.00022258805927027799  * (buffer[22] + buffer[170]) +
            -0.00013323230495695704  * (buffer[23] + buffer[169]) +
             0.00016182578767055206  * (buffer[25] + buffer[167]) +
             0.00032846175385096581  * (buffer[26] + buffer[166]) +
             0.00047045611576184863  * (buffer[27] + buffer[165]) +
             0.00055713851457530944  * (buffer[28] + buffer[164]) +
             0.00056212565121518726  * (buffer[29] + buffer[163]) +
             0.00046901918553962478  * (buffer[30] + buffer[162]) +
             0.00027624866838952986  * (buffer[31] + buffer[161]) +
            -0.00032564179486838622  * (buffer[33] + buffer[159]) +
            -0.00065182310286710388  * (buffer[34] + buffer[158]) +
            -0.00092127787309319298  * (buffer[35] + buffer[157]) +
            -0.0010772534348943575   * (buffer[36] + buffer[156]) +
            -0.0010737727700273478   * (buffer[37] + buffer[155]) +
            -0.00088556645390392634  * (buffer[38] + buffer[154]) +
            -0.00051581896090765534  * (buffer[39] + buffer[153]) +
             0.00059548767193795277  * (buffer[41] + buffer[151]) +
             0.0011803558710661009   * (buffer[42] + buffer[150]) +
             0.0016527320270369871   * (buffer[43] + buffer[149]) +
             0.0019152679330965555   * (buffer[44] + buffer[148]) +
             0.0018927324805381538   * (buffer[45] + buffer[147]) +
             0.0015481870327877937   * (buffer[46] + buffer[146]) +
             0.00089470695834941306  * (buffer[47] + buffer[145]) +
            -0.0010178225878206125   * (buffer[49] + buffer[143]) +
            -0.0020037400552054292   * (buffer[50] + buffer[142]) +
            -0.0027874356824117317   * (buffer[51] + buffer[141]) +
            -0.003210329988021943    * (buffer[52] + buffer[140]) +
            -0.0031540624117984395   * (buffer[53] + buffer[139]) +
            -0.0025657163651900345   * (buffer[54] + buffer[138]) +
            -0.0014750752642111449   * (buffer[55] + buffer[137]) +
             0.0016624165446378462   * (buffer[57] + buffer[135]) +
             0.0032591192839069179   * (buffer[58] + buffer[134]) +
             0.0045165685815867747   * (buffer[59] + buffer[133]) +
             0.0051838984346123896   * (buffer[60] + buffer[132]) +
             0.0050774264697459933   * (buffer[61] + buffer[131]) +
             0.0041192521414141585   * (buffer[62] + buffer[130]) +
             0.0023628575417966491   * (buffer[63] + buffer[129]) +
            -0.0026543507866759182   * (buffer[65] + buffer[127]) +
            -0.0051990251084333425   * (buffer[66] + buffer[126]) +
            -0.0072020238234656924   * (buffer[67] + buffer[125]) +
            -0.0082672928192007358   * (buffer[68] + buffer[124]) +
            -0.0081033739572956287   * (buffer[69] + buffer[123]) +
            -0.006583111539570221    * (buffer[70] + buffer[122]) +
            -0.0037839040415292386   * (buffer[71] + buffer[121]) +
             0.0042781252851152507   * (buffer[73] + buffer[119]) +
             0.0084176358598320178   * (buffer[74] + buffer[118]) +
             0.01172566057463055     * (buffer[75] + buffer[117]) +
             0.013550476647788672    * (buffer[76] + buffer[116]) +
             0.013388189369997496    * (buffer[77] + buffer[115]) +
             0.010979501242341259    * (buffer[78] + buffer[114]) +
             0.006381274941685413    * (buffer[79] + buffer[113]) +
            -0.007421229604153888    * (buffer[81] + buffer[111]) +
            -0.01486456304340213     * (buffer[82] + buffer[110]) +
            -0.021143584622178104    * (buffer[83] + buffer[109]) +
            -0.02504275058758609     * (buffer[84] + buffer[108]) +
            -0.025473530942547201    * (buffer[85] + buffer[107]) +
            -0.021627310017882196    * (buffer[86] + buffer[106]) +
            -0.013104323383225543    * (buffer[87] + buffer[105]) +
             0.017065133989980476    * (buffer[89] + buffer[103]) +
             0.036978919264451952    * (buffer[90] + buffer[102]) +
             0.05823318062093958     * (buffer[91] + buffer[101]) +
             0.079072012081405949    * (buffer[92] + buffer[100]) +
             0.097675998716952317    * (buffer[93] + buffer[99])  +
             0.11236045936950932     * (buffer[94] + buffer[98])  +
             0.12176343577287731     * (buffer[95] + buffer[97])  +
             0.125                   * buffer[96];

        // Copy first chunk to last chunk
        let (mid, end) = buffer.split_at_mut(FIR_SIZE - DECIMATE_FACTOR);
        let (start, _) = mid.split_at(DECIMATE_FACTOR);
        end.copy_from_slice(start);

        result
    }
}
