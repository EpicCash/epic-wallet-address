use super::OutputData;
use grin_util::secp::pedersen::Commitment;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputCommitMapping {
	/// Output Data
	pub output: OutputData,
	/// The commit
	pub commit: Commitment,
}
