mod arch;
mod mds;
mod rpo;
mod rpx;

use core::ops::Range;

use winter_math::FieldElement;

pub use self::{
	arch::optimized::{add_constants_and_apply_inv_sbox, add_constants_and_apply_sbox},
	rpo::{Rpo256, RpoDigest, RpoDigestError},
	rpx::{Rpx256, RpxDigest, RpxDigestError},
};
use crate::{Felt, StarkField};

const NUM_ROUNDS: usize = 7;

const STATE_WIDTH: usize = 12;
const RATE_RANGE: Range<usize> = 4..12;
const RATE_WIDTH: usize = RATE_RANGE.end - RATE_RANGE.start;

const INPUT1_RANGE: Range<usize> = 4..8;
const INPUT2_RANGE: Range<usize> = 8..12;

const CAPACITY_RANGE: Range<usize> = 0..4;

const DIGEST_RANGE: Range<usize> = 4..8;
const DIGEST_SIZE: usize = DIGEST_RANGE.end - DIGEST_RANGE.start;

const DIGEST_BYTES: usize = 32;

const BINARY_CHUNK_SIZE: usize = 7;

#[cfg(test)]
const ALPHA: u64 = 7;
#[cfg(test)]
const INV_ALPHA: u64 = 10_540_996_611_094_048_183;

const ARK1: [[Felt; STATE_WIDTH]; NUM_ROUNDS] = [
	[
		Felt::new(5_789_762_306_288_267_392),
		Felt::new(6_522_564_764_413_701_783),
		Felt::new(17_809_893_479_458_208_203),
		Felt::new(107_145_243_989_736_508),
		Felt::new(6_388_978_042_437_517_382),
		Felt::new(15_844_067_734_406_016_715),
		Felt::new(9_975_000_513_555_218_239),
		Felt::new(3_344_984_123_768_313_364),
		Felt::new(9_959_189_626_657_347_191),
		Felt::new(12_960_773_468_763_563_665),
		Felt::new(9_602_914_297_752_488_475),
		Felt::new(16_657_542_370_200_465_908),
	],
	[
		Felt::new(12_987_190_162_843_096_997),
		Felt::new(653_957_632_802_705_281),
		Felt::new(4_441_654_670_647_621_225),
		Felt::new(4_038_207_883_745_915_761),
		Felt::new(5_613_464_648_874_830_118),
		Felt::new(13_222_989_726_778_338_773),
		Felt::new(3_037_761_201_230_264_149),
		Felt::new(16_683_759_727_265_180_203),
		Felt::new(8_337_364_536_491_240_715),
		Felt::new(3_227_397_518_293_416_448),
		Felt::new(8_110_510_111_539_674_682),
		Felt::new(2_872_078_294_163_232_137),
	],
	[
		Felt::new(18_072_785_500_942_327_487),
		Felt::new(6_200_974_112_677_013_481),
		Felt::new(17_682_092_219_085_884_187),
		Felt::new(10_599_526_828_986_756_440),
		Felt::new(975_003_873_302_957_338),
		Felt::new(8_264_241_093_196_931_281),
		Felt::new(10_065_763_900_435_475_170),
		Felt::new(2_181_131_744_534_710_197),
		Felt::new(6_317_303_992_309_418_647),
		Felt::new(1_401_440_938_888_741_532),
		Felt::new(8_884_468_225_181_997_494),
		Felt::new(13_066_900_325_715_521_532),
	],
	[
		Felt::new(5_674_685_213_610_121_970),
		Felt::new(5_759_084_860_419_474_071),
		Felt::new(13_943_282_657_648_897_737),
		Felt::new(1_352_748_651_966_375_394),
		Felt::new(17_110_913_224_029_905_221),
		Felt::new(1_003_883_795_902_368_422),
		Felt::new(4_141_870_621_881_018_291),
		Felt::new(8_121_410_972_417_424_656),
		Felt::new(14_300_518_605_864_919_529),
		Felt::new(13_712_227_150_607_670_181),
		Felt::new(17_021_852_944_633_065_291),
		Felt::new(6_252_096_473_787_587_650),
	],
	[
		Felt::new(4_887_609_836_208_846_458),
		Felt::new(3_027_115_137_917_284_492),
		Felt::new(9_595_098_600_469_470_675),
		Felt::new(10_528_569_829_048_484_079),
		Felt::new(7_864_689_113_198_939_815),
		Felt::new(17_533_723_827_845_969_040),
		Felt::new(5_781_638_039_037_710_951),
		Felt::new(17_024_078_752_430_719_006),
		Felt::new(109_659_393_484_013_511),
		Felt::new(7_158_933_660_534_805_869),
		Felt::new(2_955_076_958_026_921_730),
		Felt::new(7_433_723_648_458_773_977),
	],
	[
		Felt::new(16_308_865_189_192_447_297),
		Felt::new(11_977_192_855_656_444_890),
		Felt::new(12_532_242_556_065_780_287),
		Felt::new(14_594_890_931_430_968_898),
		Felt::new(7_291_784_239_689_209_784),
		Felt::new(5_514_718_540_551_361_949),
		Felt::new(10_025_733_853_830_934_803),
		Felt::new(7_293_794_580_341_021_693),
		Felt::new(6_728_552_937_464_861_756),
		Felt::new(6_332_385_040_983_343_262),
		Felt::new(13_277_683_694_236_792_804),
		Felt::new(2_600_778_905_124_452_676),
	],
	[
		Felt::new(7_123_075_680_859_040_534),
		Felt::new(1_034_205_548_717_903_090),
		Felt::new(7_717_824_418_247_931_797),
		Felt::new(3_019_070_937_878_604_058),
		Felt::new(11_403_792_746_066_867_460),
		Felt::new(10_280_580_802_233_112_374),
		Felt::new(337_153_209_462_421_218),
		Felt::new(13_333_398_568_519_923_717),
		Felt::new(3_596_153_696_935_337_464),
		Felt::new(8_104_208_463_525_993_784),
		Felt::new(14_345_062_289_456_085_693),
		Felt::new(17_036_731_477_169_661_256),
	],
];

const ARK2: [[Felt; STATE_WIDTH]; NUM_ROUNDS] = [
	[
		Felt::new(6_077_062_762_357_204_287),
		Felt::new(15_277_620_170_502_011_191),
		Felt::new(5_358_738_125_714_196_705),
		Felt::new(14_233_283_787_297_595_718),
		Felt::new(13_792_579_614_346_651_365),
		Felt::new(11_614_812_331_536_767_105),
		Felt::new(14_871_063_686_742_261_166),
		Felt::new(10_148_237_148_793_043_499),
		Felt::new(4_457_428_952_329_675_767),
		Felt::new(15_590_786_458_219_172_475),
		Felt::new(10_063_319_113_072_092_615),
		Felt::new(14_200_078_843_431_360_086),
	],
	[
		Felt::new(6_202_948_458_916_099_932),
		Felt::new(17_690_140_365_333_231_091),
		Felt::new(3_595_001_575_307_484_651),
		Felt::new(373_995_945_117_666_487),
		Felt::new(1_235_734_395_091_296_013),
		Felt::new(14_172_757_457_833_931_602),
		Felt::new(707_573_103_686_350_224),
		Felt::new(15_453_217_512_188_187_135),
		Felt::new(219_777_875_004_506_018),
		Felt::new(17_876_696_346_199_469_008),
		Felt::new(17_731_621_626_449_383_378),
		Felt::new(2_897_136_237_748_376_248),
	],
	[
		Felt::new(8_023_374_565_629_191_455),
		Felt::new(15_013_690_343_205_953_430),
		Felt::new(4_485_500_052_507_912_973),
		Felt::new(12_489_737_547_229_155_153),
		Felt::new(9_500_452_585_969_030_576),
		Felt::new(2_054_001_340_201_038_870),
		Felt::new(12_420_704_059_284_934_186),
		Felt::new(355_990_932_618_543_755),
		Felt::new(9_071_225_051_243_523_860),
		Felt::new(12_766_199_826_003_448_536),
		Felt::new(9_045_979_173_463_556_963),
		Felt::new(12_934_431_667_190_679_898),
	],
	[
		Felt::new(18_389_244_934_624_494_276),
		Felt::new(16_731_736_864_863_925_227),
		Felt::new(4_440_209_734_760_478_192),
		Felt::new(17_208_448_209_698_888_938),
		Felt::new(8_739_495_587_021_565_984),
		Felt::new(17_000_774_922_218_161_967),
		Felt::new(13_533_282_547_195_532_087),
		Felt::new(525_402_848_358_706_231),
		Felt::new(16_987_541_523_062_161_972),
		Felt::new(5_466_806_524_462_797_102),
		Felt::new(14_512_769_585_918_244_983),
		Felt::new(10_973_956_031_244_051_118),
	],
	[
		Felt::new(6_982_293_561_042_362_913),
		Felt::new(14_065_426_295_947_720_331),
		Felt::new(16_451_845_770_444_974_180),
		Felt::new(7_139_138_592_091_306_727),
		Felt::new(9_012_006_439_959_783_127),
		Felt::new(14_619_614_108_529_063_361),
		Felt::new(1_394_813_199_588_124_371),
		Felt::new(4_635_111_139_507_788_575),
		Felt::new(16_217_473_952_264_203_365),
		Felt::new(10_782_018_226_466_330_683),
		Felt::new(6_844_229_992_533_662_050),
		Felt::new(7_446_486_531_695_178_711),
	],
	[
		Felt::new(3_736_792_340_494_631_448),
		Felt::new(577_852_220_195_055_341),
		Felt::new(6_689_998_335_515_779_805),
		Felt::new(13_886_063_479_078_013_492),
		Felt::new(14_358_505_101_923_202_168),
		Felt::new(7_744_142_531_772_274_164),
		Felt::new(16_135_070_735_728_404_443),
		Felt::new(12_290_902_521_256_031_137),
		Felt::new(12_059_913_662_657_709_804),
		Felt::new(16_456_018_495_793_751_911),
		Felt::new(4_571_485_474_751_953_524),
		Felt::new(17_200_392_109_565_783_176),
	],
	[
		Felt::new(17_130_398_059_294_018_733),
		Felt::new(519_782_857_322_261_988),
		Felt::new(9_625_384_390_925_085_478),
		Felt::new(1_664_893_052_631_119_222),
		Felt::new(7_629_576_092_524_553_570),
		Felt::new(3_485_239_601_103_661_425),
		Felt::new(9_755_891_797_164_033_838),
		Felt::new(15_218_148_195_153_269_027),
		Felt::new(16_460_604_813_734_957_368),
		Felt::new(9_643_968_136_937_729_763),
		Felt::new(3_611_348_709_641_382_851),
		Felt::new(18_256_379_591_337_759_196),
	],
];

#[inline]
fn apply_sbox(state: &mut [Felt; STATE_WIDTH]) {
	state[0] = state[0].exp7();
	state[1] = state[1].exp7();
	state[2] = state[2].exp7();
	state[3] = state[3].exp7();
	state[4] = state[4].exp7();
	state[5] = state[5].exp7();
	state[6] = state[6].exp7();
	state[7] = state[7].exp7();
	state[8] = state[8].exp7();
	state[9] = state[9].exp7();
	state[10] = state[10].exp7();
	state[11] = state[11].exp7();
}

#[inline]
fn apply_inv_sbox(state: &mut [Felt; STATE_WIDTH]) {
	fn exp_acc<B: StarkField, const N: usize, const M: usize>(
		base: [B; N],
		tail: [B; N],
	) -> [B; N] {
		let mut result = base;
		for _ in 0..M {
			result.iter_mut().for_each(|r| *r = r.square());
		}
		result.iter_mut().zip(tail).for_each(|(r, t)| *r *= t);
		result
	}

	let mut t1 = *state;
	t1.iter_mut().for_each(|t| *t = t.square());

	let mut t2 = t1;
	t2.iter_mut().for_each(|t| *t = t.square());

	let t3 = exp_acc::<Felt, STATE_WIDTH, 3>(t2, t2);
	let t4 = exp_acc::<Felt, STATE_WIDTH, 6>(t3, t3);
	let t5 = exp_acc::<Felt, STATE_WIDTH, 12>(t4, t4);
	let t6 = exp_acc::<Felt, STATE_WIDTH, 6>(t5, t3);
	let t7 = exp_acc::<Felt, STATE_WIDTH, 31>(t6, t6);

	for (i, s) in state.iter_mut().enumerate() {
		let a = (t7[i].square() * t6[i]).square().square();
		let b = t1[i] * t2[i] * *s;
		*s = a * b;
	}
}

#[allow(clippy::trivially_copy_pass_by_ref)]
#[inline]
fn add_constants(state: &mut [Felt; STATE_WIDTH], ark: &[Felt; STATE_WIDTH]) {
	state.iter_mut().zip(ark).for_each(|(s, &k)| *s += k);
}
