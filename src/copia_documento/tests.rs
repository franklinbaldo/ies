//! RED/GREEN da prévia de `documento salvar` pelo par `DOC.vN.html` /
//! `DOC.v(N+1).html` (#207), sem base nem manifesto ocultos.
use super::*;
use crate::foto_documento::{gravar, tests::pasta};
use std::collections::BTreeMap;

fn v(n: u32) -> VersaoDocumento {
    VersaoDocumento::nova(n).unwrap()
}

fn doc() -> NumeroDocumento {
    NumeroDocumento::novo("0011519").unwrap()
}

const FOTO: &str = "<html><head><title>SEI/X - 0011519 - Despacho</title></head>\n<body>\n<p>antigo</p>\n</body></html>";
const EDICAO: &str = "<html><head><title>SEI/X - 0011519 - Despacho</title></head>\n<body>\n<p>novo</p>\n</body></html>";

/// Grava a foto vN e devolve o registro com o SHA dela, como o índice.
fn com_foto(pasta: &Path, n: u32) -> BTreeMap<u32, String> {
    let foto = gravar(pasta, "0011519", v(n), FOTO).unwrap();
    BTreeMap::from([(n, foto.sha256)])
}

fn editar(pasta: &Path, n: u32, corpo: &str) {
    use std::io::Write;
    let mut opcoes = fs::OpenOptions::new();
    opcoes.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opcoes.mode(0o600);
    }
    opcoes
        .open(pasta.join(format!("0011519.v{n}.html")))
        .unwrap()
        .write_all(corpo.as_bytes())
        .unwrap();
}

fn previa(
    pasta: &Path,
    registro: &BTreeMap<u32, String>,
    n: Option<u32>,
) -> Result<PreviaCopia, ErroCopia> {
    let consulta = |versao: VersaoDocumento| registro.get(&versao.numero()).cloned();
    previa_local(pasta, &doc(), n.map(v), &consulta)
}

#[test]
fn par_vn_e_v_n_mais_1_publica_o_trecho_sem_o_texto() {
    let pasta = pasta();
    let registro = com_foto(&pasta, 2);
    editar(&pasta, 3, EDICAO);
    for pedida in [None, Some(2)] {
        let p = previa(&pasta, &registro, pedida).unwrap();
        assert_eq!((p.versao_base, p.versao_editada), (2, 3), "{pedida:?}");
        assert_eq!((p.bytes_base, p.bytes_editada), (FOTO.len(), EDICAO.len()));
        assert!(!p.escrita_remota);
        assert_eq!(p.trecho_divergente.unwrap().base_inicio, 2);
        let json = serde_json::to_string(&p).unwrap();
        assert!(!json.contains("novo") && !json.contains("<p>"));
    }
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn so_a_foto_sem_edicao_e_edicao_igual_sao_recusadas() {
    let pasta = pasta();
    let registro = com_foto(&pasta, 2);
    assert_eq!(
        previa(&pasta, &registro, None),
        Err(ErroCopia::EdicaoAusente)
    );
    assert_eq!(
        previa(&pasta, &registro, Some(2)),
        Err(ErroCopia::EdicaoAusente)
    );
    editar(&pasta, 3, FOTO);
    assert_eq!(
        previa(&pasta, &registro, None),
        Err(ErroCopia::EdicaoIgualABase)
    );
    assert_eq!(
        ErroCopia::EdicaoIgualABase.subcodigo(),
        "versao_editada_igual_a_base"
    );
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn base_alterada_ou_nao_registrada_e_recusada() {
    let pasta = pasta();
    let registro = com_foto(&pasta, 2);
    editar(&pasta, 3, EDICAO);
    // A foto vN foi mexida depois de gravada: o SHA não bate.
    let outro = BTreeMap::from([(2, "0".repeat(64))]);
    assert_eq!(previa(&pasta, &outro, None), Err(ErroCopia::BaseDivergente));
    assert_eq!(
        previa(&pasta, &BTreeMap::new(), None),
        Err(ErroCopia::BaseNaoRegistrada)
    );
    // `--versao` sem a foto daquela versão.
    assert_eq!(
        previa(&pasta, &registro, Some(5)),
        Err(ErroCopia::BaseAusente)
    );
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn sem_arquivos_e_duas_grafias_sao_recusados() {
    let pasta = pasta();
    assert_eq!(
        previa(&pasta, &BTreeMap::new(), None),
        Err(ErroCopia::BaseAusente)
    );
    editar(&pasta, 2, FOTO);
    fs::write(pasta.join("0011519.v02.html"), "x").unwrap();
    assert_eq!(
        previa(&pasta, &BTreeMap::new(), None),
        Err(ErroCopia::VersaoDuplicada)
    );
    fs::remove_dir_all(pasta).unwrap();
}

#[cfg(unix)]
#[test]
fn edicao_por_link_simbolico_e_recusada() {
    use std::os::unix::fs::symlink;
    let pasta = pasta();
    let registro = com_foto(&pasta, 2);
    symlink("/etc/hostname", pasta.join("0011519.v3.html")).unwrap();
    assert_eq!(
        previa(&pasta, &registro, Some(2)),
        Err(ErroCopia::LinkSimbolico)
    );
    fs::remove_dir_all(pasta).unwrap();
}
