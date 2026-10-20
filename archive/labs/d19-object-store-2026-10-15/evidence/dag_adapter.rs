//! Compiled against the actual #2489 reader source b32651b82; no peer code edits.
use uor_r4_object_store::{RecordBackend, Store};
use uor_r4_recon_dag::{Backend,Error,Id,Node};
struct StoreAdapter<'a>(&'a Store);
impl Backend for StoreAdapter<'_> {
    fn node(&self,id:Id)->Result<Node,Error> {
        let record=self.0.fetch(id).map_err(|e|Error::Io(e.to_string()))?.ok_or(Error::Missing(id))?;
        if record.id()!=id {return Err(Error::Integrity);}
        // The reader independently verifies shape, scale, and hash after this conversion.
        Ok(Node {scale:record.scale(),data:record.payload().to_vec()})
    }
    fn stored(&self)->usize {self.0.record_count()}
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let mut store=Store::new(Default::default());
    let bytes=b"a repeated exact quotation. a repeated exact quotation. \0\0";
    let folded=store.fold(bytes)?;
    let manifest=uor_r4_recon_dag::Manifest {len:usize::try_from(folded.manifest.byte_len())?,roots:folded.manifest.roots().to_vec()};
    let adapter=StoreAdapter(&store);
    // Reader Error lacks std::error::Error at the pinned head; preserve debug identity.
    let reader_error=|e|std::io::Error::other(format!("{e:?}"));
    let full=uor_r4_recon_dag::read_whole_object(&manifest,&adapter,folded.manifest.whole_sha256()).map_err(reader_error)?;
    assert_eq!(full.bytes,bytes);assert!(full.whole_object_verified());
    let part=uor_r4_recon_dag::read_range(&manifest,&adapter,3,17).map_err(reader_error)?;
    assert_eq!(part.bytes,&bytes[3..17]);assert!(!part.whole_object_verified());
    assert!(matches!(adapter.node([0;12]),Err(Error::Missing(_))));
    println!("PASS: actual reader Backend::node/stored adapter; exact whole/range bytes; missing mapping; integrity types");
    Ok(())
}
