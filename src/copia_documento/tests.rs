//! RED/GREEN da prévia de `documento salvar` pelo par `DOC.vN.html` /
//! `DOC.v(N+1).html` (#207), offline: sem base oculta, manifesto, índice
//! ou configuração. A base não é verificada aqui.
use super::*;
use crate::foto_documento::{gravar, tests::pasta};

fn v(n: u32) -> VersaoDocumento {
    VersaoDocumento::nova(n).unwrap()
}

fn doc() -> NumeroDocumento {
    NumeroDocumento::novo("0011519").unwrap()
}

const FOTO: &str = "<html><head><title>SEI/X - 0011519 - Despacho</title></head>\n<body>\n<p>antigo</p>\n</body></html>";
const EDICAO: &str = "<html><head><title>SEI/X - 0011519 - Despacho</title></head>\n<body>\n<p>novo</p>\n</body></html>";

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

fn previa(pasta: &Path, n: Option<u32>) -> Result<PreviaCopia, ErroCopia> {
    previa_local(pasta, &doc(), n.map(v))
}

/// A foto vN como `--saida` a grava.
fn foto(pasta: &Path, n: u32) {
    gravar(pasta, "0011519", v(n), FOTO).unwrap();
}

#[test]
fn par_vn_e_v_n_mais_1_publica_o_trecho_sem_o_texto() {
    let pasta = pasta();
    foto(&pasta, 2);
    editar(&pasta, 3, EDICAO);
    for pedida in [None, Some(2)] {
        let p = previa(&pasta, pedida).unwrap();
        assert_eq!((p.versao_base, p.versao_editada), (2, 3), "{pedida:?}");
        assert_eq!((p.bytes_base, p.bytes_editada), (FOTO.len(), EDICAO.len()));
        assert!(!p.escrita_remota);
        assert_eq!(p.trecho_divergente.unwrap().base_inicio, 2);
        let json = serde_json::to_string(&p).unwrap();
        assert!(!json.contains("novo") && !json.contains("<p>"));
        assert!(json.contains("\"base_verificada\":false"), "{json}");
    }
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn base_qualquer_e_aceita_sem_verificacao() {
    // Offline não há com que conferir a base: um `DOC.v2.html` que não
    // veio de `--saida` (ou foi alterado) vale como base na prévia. Quem
    // confere é a releitura (`foto_local_divergente`) ou o salvar remoto.
    let pasta = pasta();
    editar(&pasta, 2, "<p>escrito à mão</p>");
    editar(&pasta, 3, EDICAO);
    let p = previa(&pasta, None).unwrap();
    assert_eq!((p.versao_base, p.versao_editada), (2, 3));
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn so_a_foto_sem_edicao_e_edicao_igual_sao_recusadas() {
    let pasta = pasta();
    foto(&pasta, 2);
    assert_eq!(previa(&pasta, None), Err(ErroCopia::VersaoEditadaAusente));
    assert_eq!(
        previa(&pasta, Some(2)),
        Err(ErroCopia::VersaoEditadaAusente)
    );
    editar(&pasta, 3, FOTO);
    assert_eq!(previa(&pasta, None), Err(ErroCopia::EdicaoIgualABase));
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn base_ausente_e_recusada() {
    let pasta = pasta();
    assert_eq!(previa(&pasta, None), Err(ErroCopia::BaseVersaoAusente));
    editar(&pasta, 3, EDICAO);
    // `--versao` sem a base daquela versão.
    assert_eq!(previa(&pasta, Some(2)), Err(ErroCopia::BaseVersaoAusente));
    assert_eq!(previa(&pasta, Some(5)), Err(ErroCopia::BaseVersaoAusente));
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn duas_grafias_sao_recusadas() {
    let pasta = pasta();
    editar(&pasta, 2, FOTO);
    fs::write(pasta.join("0011519.v02.html"), "x").unwrap();
    assert_eq!(previa(&pasta, None), Err(ErroCopia::VersaoLocalRepetida));
    fs::remove_dir_all(pasta).unwrap();
}

#[cfg(unix)]
#[test]
fn edicao_por_link_simbolico_e_recusada() {
    use std::os::unix::fs::symlink;
    let pasta = pasta();
    foto(&pasta, 2);
    symlink("/etc/hostname", pasta.join("0011519.v3.html")).unwrap();
    assert_eq!(previa(&pasta, Some(2)), Err(ErroCopia::LinkSimbolico));
    fs::remove_dir_all(pasta).unwrap();
}
