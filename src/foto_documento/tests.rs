//! RED/GREEN da foto `DOC.vN.html` (#207) com HTML sintético na forma do
//! `documento_visualizar` servido (título `SEI/… - DOC - Tipo`, rodapé).
use super::*;
use crate::rodape_do_documento::testes::rodape;

fn v(n: u32) -> VersaoDocumento {
    VersaoDocumento::nova(n).unwrap()
}

fn html(numero: &str, texto: &str, versao: Option<u32>) -> String {
    let rodape = versao.map_or_else(String::new, |n| rodape(&n.to_string()));
    format!(
        "<html><head><title>SEI/EXEMPLO - {numero} - Despacho</title></head>\
         <body><p>{texto}</p>{rodape}</body></html>"
    )
}

pub(crate) fn pasta() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ies-foto-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn nomes(pasta: &Path) -> Vec<String> {
    let mut nomes: Vec<String> = fs::read_dir(pasta)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    nomes.sort();
    nomes
}

#[test]
fn grava_so_doc_vn_html_privado_sem_txt_nem_oculto() {
    let pasta = pasta();
    let corpo = html("0011519", "Atual.", Some(2));
    let foto = gravar(&pasta, "0011519", v(2), &corpo).unwrap();
    assert_eq!(
        foto,
        FotoGravada {
            nome: "0011519.v2.html".into(),
            bytes: corpo.len(),
            sha256: sha256(corpo.as_bytes()),
            gravado: true,
        }
    );
    assert_eq!(nomes(&pasta), ["0011519.v2.html"]);
    assert_eq!(fs::read_to_string(pasta.join(&foto.nome)).unwrap(), corpo);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let modo = fs::metadata(pasta.join(&foto.nome))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(modo & 0o777, 0o600);
    }
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn mesma_foto_de_novo_e_no_op_e_diferente_e_recusa() {
    let pasta = pasta();
    let corpo = html("0011519", "Atual.", Some(2));
    gravar(&pasta, "0011519", v(2), &corpo).unwrap();
    let antes = fs::metadata(pasta.join("0011519.v2.html"))
        .unwrap()
        .modified()
        .unwrap();
    let de_novo = gravar(&pasta, "0011519", v(2), &corpo).unwrap();
    assert!(!de_novo.gravado);
    assert_eq!(de_novo.sha256, sha256(corpo.as_bytes()));
    let depois = fs::metadata(pasta.join("0011519.v2.html"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(antes, depois);
    let outro = html("0011519", "Mudou.", Some(2));
    assert_eq!(
        gravar(&pasta, "0011519", v(2), &outro),
        Err(ErroCopia::FotoDivergente)
    );
    assert_eq!(
        planejar(&pasta, "0011519", v(2), &outro),
        Err(ErroCopia::FotoDivergente)
    );
    assert_eq!(ErroCopia::FotoDivergente.subcodigo(), "foto_divergente");
    assert_eq!(
        fs::read_to_string(pasta.join("0011519.v2.html")).unwrap(),
        corpo
    );
    assert_eq!(nomes(&pasta), ["0011519.v2.html"]);
    fs::remove_dir_all(pasta).unwrap();
}

#[cfg(unix)]
#[test]
fn nao_segue_link_simbolico_no_nome_da_foto() {
    use std::os::unix::fs::symlink;
    let pasta = pasta();
    let alvo = pasta.join("alvo");
    fs::write(&alvo, "x").unwrap();
    symlink(&alvo, pasta.join("0011519.v2.html")).unwrap();
    let corpo = html("0011519", "Atual.", Some(2));
    assert_eq!(
        gravar(&pasta, "0011519", v(2), &corpo),
        Err(ErroCopia::LinkSimbolico)
    );
    assert_eq!(fs::read_to_string(&alvo).unwrap(), "x");
    fs::remove_dir_all(pasta).unwrap();
}

/// `--saida atalho` em que `atalho` é link para um diretório real:
/// aceito (conferido e gravado no diretório real), e o modo do diretório
/// da pessoa não é conferido (gravável pelo grupo ou por todos).
#[cfg(unix)]
#[test]
fn saida_por_link_simbolico_e_gravavel_por_outros_e_aceita() {
    use std::os::unix::fs::PermissionsExt;
    let raiz = pasta();
    let real = raiz.join("real");
    fs::create_dir(&real).unwrap();
    let atalho = raiz.join("atalho");
    std::os::unix::fs::symlink(&real, &atalho).unwrap();
    let corpo = html("0011519", "Atual.", Some(2));
    assert_eq!(conferir_antes_da_rede(&atalho, &[], Some(v(2))), Ok(()));
    assert!(gravar(&atalho, "0011519", v(2), &corpo).unwrap().gravado);
    assert_eq!(nomes(&real), ["0011519.v2.html"]);
    for (bits, n) in [(0o775, 3), (0o777, 4)] {
        fs::set_permissions(&real, fs::Permissions::from_mode(bits)).unwrap();
        assert_eq!(conferir_antes_da_rede(&atalho, &[], None), Ok(()));
        let corpo = html("0011519", "Outra.", Some(n));
        assert!(gravar(&atalho, "0011519", v(n), &corpo).unwrap().gravado);
    }
    fs::remove_dir_all(raiz).unwrap();
}

#[test]
fn com_versao_a_foto_presente_e_conferida_antes_da_rede() {
    let pasta = pasta();
    let numeros = [NumeroDocumento::novo("0011519").unwrap()];
    assert_eq!(conferir_antes_da_rede(&pasta, &numeros, Some(v(1))), Ok(()));
    gravar(
        &pasta,
        "0011519",
        v(1),
        &html("0011519", "Antiga.", Some(1)),
    )
    .unwrap();
    assert_eq!(conferir_antes_da_rede(&pasta, &numeros, Some(v(1))), Ok(()));
    // Outro número SEI ou rodapé de outra versão sob o nome v3: recusa.
    gravar(&pasta, "0011519", v(3), &html("0099999", "Outro.", None)).unwrap();
    assert_eq!(
        conferir_antes_da_rede(&pasta, &numeros, Some(v(3))),
        Err(ErroCopia::FotoDivergente)
    );
    gravar(
        &pasta,
        "0011519",
        v(4),
        &html("0011519", "Outra versão.", Some(5)),
    )
    .unwrap();
    assert_eq!(
        conferir_antes_da_rede(&pasta, &numeros, Some(v(4))),
        Err(ErroCopia::FotoDivergente)
    );
    // Sem `--versao` o nome não é conhecido: só o diretório.
    assert_eq!(conferir_antes_da_rede(&pasta, &numeros, None), Ok(()));
    let ausente = pasta.join("nao-existe");
    assert_eq!(
        conferir_antes_da_rede(&ausente, &numeros, None).map_err(ErroCopia::subcodigo),
        Err("diretorio_ausente")
    );
    fs::remove_dir_all(pasta).unwrap();
}
