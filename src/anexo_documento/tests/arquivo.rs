//! Regressões locais da gravação do anexo em `--saida DIR`.
//! Acervo Web e SQLite não exercitam falhas do filesystem do usuário.
use super::*;

#[cfg(unix)]
#[test]
fn pasta_do_anexo_com_ancestral_regular_preserva_causa_e_nao_grava() {
    let raiz = pasta();
    let sentinela = raiz.join("arquivo-regular");
    std::fs::write(&sentinela, b"intacto").unwrap();
    let destino = sentinela.join("subdiretorio");

    assert_eq!(
        ler_anexo(
            pdf(&[]),
            "0000101",
            LimiteDoAnexo::default(),
            DestinoDoAnexo::Diretorio(&destino)
        ),
        Err(ErroAnexo::GravacaoDoParcialFalhou(
            FalhaDaGravacao::DiretorioNaoConsultado(io::ErrorKind::NotADirectory)
        ))
    );
    assert_eq!(std::fs::read(&sentinela).unwrap(), b"intacto");
    assert_eq!(std::fs::read_dir(&raiz).unwrap().count(), 1);
    std::fs::remove_dir_all(raiz).unwrap();
}

#[test]
fn pdf_com_saida_vai_para_parcial_0600_e_so_conclui_sem_sobrescrever() {
    let dir = pasta();
    let mut a = ler_anexo(
        pdf(&[]),
        "0000101",
        LimiteDoAnexo::default(),
        DestinoDoAnexo::Diretorio(&dir),
    )
    .unwrap();
    let arquivo = a.tirar_arquivo().expect("parcial");
    assert_eq!(arquivo.nome(), "0000101.pdf");
    let parciais: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().collect();
    assert_eq!(parciais.len(), 1);
    let nome = parciais[0].file_name().into_string().unwrap();
    assert!(nome.starts_with(".0000101.pdf.parcial-"), "{nome}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let modo = parciais[0].metadata().unwrap().permissions().mode();
        assert_eq!(modo & 0o777, 0o600);
    }
    arquivo.concluir().unwrap();
    assert_eq!(
        std::fs::read(dir.join("0000101.pdf")).unwrap(),
        b"%PDF-1.7 sintetico"
    );
    assert_eq!(
        std::fs::read_dir(&dir).unwrap().count(),
        1,
        "parcial removido"
    );
    // De novo: o destino existe e é recusado antes do corpo.
    let r = RespostaBinaria {
        status: 200,
        cabecalhos: vec![("content-type".into(), "application/pdf".into())],
        corpo: NaoLer,
    };
    assert_eq!(
        ler_anexo(
            r,
            "0000101",
            LimiteDoAnexo::default(),
            DestinoDoAnexo::Diretorio(&dir)
        ),
        Err(ErroAnexo::ArquivoJaExiste)
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn saida_por_link_e_gravavel_por_outros_e_aceita() {
    // `--saida DIR` é da pessoa: o modo do diretório é dela.
    use std::os::unix::fs::PermissionsExt;
    let raiz = pasta();
    let real = raiz.join("real");
    std::fs::create_dir(&real).unwrap();
    let atalho = raiz.join("atalho");
    std::os::unix::fs::symlink(&real, &atalho).unwrap();
    for (bits, numero) in [(0o755, "0000103"), (0o775, "0000104"), (0o777, "0000105")] {
        std::fs::set_permissions(&real, std::fs::Permissions::from_mode(bits)).unwrap();
        let mut a = ler_anexo(
            pdf(&[]),
            numero,
            LimiteDoAnexo::default(),
            DestinoDoAnexo::Diretorio(&atalho),
        )
        .unwrap();
        a.tirar_arquivo().expect("parcial").concluir().unwrap();
        assert!(real.join(format!("{numero}.pdf")).is_file(), "{bits:o}");
    }
    assert_eq!(std::fs::read_dir(&real).unwrap().count(), 3);
    std::fs::remove_dir_all(raiz).unwrap();
}

#[test]
fn parcial_abandonado_e_removido() {
    let dir = pasta();
    let a = ler_anexo(
        pdf(&[]),
        "0000102",
        LimiteDoAnexo::default(),
        DestinoDoAnexo::Diretorio(&dir),
    )
    .unwrap();
    drop(a);
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    std::fs::remove_dir_all(dir).unwrap();
}
