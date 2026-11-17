use super::*;

#[test]
fn nome_conhecido_e_desconhecido() {
    assert_eq!(
        nome(Some(2), "aabbccddeeff001122"),
        "v2-aabbccddeeff0011.html"
    );
    assert_eq!(nome(None, "aabbccddeeff001122"), "x-aabbccddeeff0011.html");
}

#[test]
fn cache_idempotente_em_xdg_temporario() {
    let home = std::env::temp_dir().join(format!("ies-cache-red-{}", std::process::id()));
    let alvo = caminho_em(
        &home,
        "sei.exemplo",
        "0011519",
        Some(2),
        "aabbccddeeff001122",
        "html",
    );
    assert_eq!(
        alvo,
        home.join("ies/sei.exemplo/0011519/v2-aabbccddeeff0011.html")
    );
    let bytes = b"<html><body>teste</body></html>";
    let alvo = gravar_em(&home, "sei.exemplo", "0011519", Some(2), "html", bytes).unwrap();
    let primeira = std::fs::metadata(&alvo).unwrap().modified().unwrap();
    let segunda = gravar_em(&home, "sei.exemplo", "0011519", Some(2), "html", bytes).unwrap();
    assert_eq!(alvo, segunda);
    assert_eq!(
        std::fs::metadata(&alvo).unwrap().modified().unwrap(),
        primeira
    );
    std::fs::remove_dir_all(home).unwrap();
}

#[test]
fn sem_cache_nao_escreve() {
    assert!(!habilitado(true, None));
    assert!(!habilitado(false, Some("0")));
    assert!(habilitado(false, None));
}

#[test]
fn externo_pdf_em_fluxo_e_copia_opcional() {
    use crate::arquivo_privado::{Parcial, Politica};
    let raiz = std::env::temp_dir().join(format!(
        "ies-cache-anexo-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("teste"),
    ));
    let destino_parcial = raiz.join("ies/sei.exemplo/0007/0007.cache.pdf");
    let mut parcial = Parcial::abrir(
        &destino_parcial,
        Politica::exclusiva()
            .diretorio_do_ies()
            .criando_diretorios(),
    )
    .unwrap();
    let bytes = b"%PDF-1.7\nTESTE";
    parcial.escrever(bytes).unwrap();
    parcial.sincronizar().unwrap();
    let hash = sha256(bytes);
    let registro = publicar_parcial(
        parcial.temporario(),
        &raiz,
        "sei.exemplo",
        "0007",
        &hash,
        "pdf",
    )
    .unwrap();
    assert_eq!(registro.versao, None);
    assert!(registro.caminho.ends_with(format!("x-{}.pdf", &hash[..16])));
    drop(parcial);
    assert_eq!(fs::read(&registro.caminho).unwrap(), bytes);
    let pasta = raiz.join("saida");
    fs::create_dir(&pasta).unwrap();
    let copia = pasta.join("0007.pdf");
    assert_eq!(
        copiar_para_saida(&registro.caminho, &copia, &hash),
        Ok(true)
    );
    assert_eq!(
        copiar_para_saida(&registro.caminho, &copia, &hash),
        Ok(false)
    );
    assert_eq!(fs::read(&copia).unwrap(), bytes);
    fs::remove_dir_all(raiz).unwrap();
}

#[test]
fn diferentes_versoes_nao_sobrescrevem() {
    let raiz = std::env::temp_dir().join(format!("ies-cache-versoes-{}", std::process::id()));
    let um = gravar_em(&raiz, "sei.exemplo", "001", Some(1), "html", b"<p>um</p>").unwrap();
    let dois = gravar_em(&raiz, "sei.exemplo", "001", Some(2), "html", b"<p>dois</p>").unwrap();
    let desconhecida = gravar_em(&raiz, "sei.exemplo", "001", None, "html", b"<p>x</p>").unwrap();
    assert_ne!(um, dois);
    assert_ne!(desconhecida, um);
    assert!(
        desconhecida
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("x-")
    );
    fs::remove_dir_all(raiz).unwrap();
}

// Apoio técnico ao BDD-60-03: a foto PDF editável do usuário não
// pode compartilhar inode com o cache privado de conteúdo observado.
#[test]
fn bdd_60_03_copia_pdf_editada_nao_adultera_cache_privado() {
    let raiz = std::env::temp_dir().join(format!(
        "ies-cache-pdf-independente-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let pdf = b"%PDF-1.7 original";
    let origem = gravar_em(&raiz, "sei.exemplo", "0007", None, "pdf", pdf).unwrap();
    let saida = raiz.join("saida");
    fs::create_dir(&saida).unwrap();
    let destino = saida.join("0007.pdf");
    let hash = sha256(pdf);
    assert_eq!(copiar_para_saida(&origem, &destino, &hash), Ok(true));
    fs::write(&destino, b"%PDF-1.7 edicao local").unwrap();
    assert_eq!(
        fs::read(&origem).unwrap(),
        pdf,
        "cache não pode ser editado pela foto"
    );
    assert_eq!(
        copiar_para_saida(&origem, &destino, &hash),
        Err(FalhaCache::ArquivoDivergente),
        "a edição local não pode ser sobrescrita"
    );
    fs::remove_dir_all(raiz).unwrap();
}

// Contraprova da #719: a consulta do caminho falhou no SO, não foi
// uma recusa genérica da gravação. Não grava nem acessa o SEI.
#[cfg(unix)]
#[test]
fn consulta_do_cache_preserva_not_a_directory() {
    let raiz =
        std::env::temp_dir().join(format!("ies-cache-causa-diretorio-{}", std::process::id()));
    fs::write(&raiz, b"arquivo, nao diretorio").unwrap();
    let causa = gravar_em(&raiz, "sei.exemplo", "001", None, "html", b"teste").unwrap_err();
    assert_eq!(
        causa,
        FalhaCache::ArquivoDoCacheInacessivel(std::io::ErrorKind::NotADirectory)
    );
    assert_eq!(fs::read(&raiz).unwrap(), b"arquivo, nao diretorio");
    fs::remove_file(raiz).unwrap();
}

// Contraprova da #719: a publicação por hard link também precisa
// preservar o motivo de E/S. Um ancestral é arquivo, não diretório.
#[cfg(unix)]
#[test]
fn publicar_parcial_preserva_erro_de_hard_link() {
    let raiz = std::env::temp_dir().join(format!(
        "ies-cache-publicar-causa-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&raiz).unwrap();
    let ancestral = raiz.join("ies");
    fs::write(&ancestral, b"sentinela").unwrap();
    let parcial = raiz.join("parcial.pdf");
    let pdf = b"%PDF-1.7 sintetico";
    fs::write(&parcial, pdf).unwrap();
    let erro =
        publicar_parcial(&parcial, &raiz, "sei.exemplo", "0007", &sha256(pdf), "pdf").unwrap_err();
    assert_eq!(
        erro,
        FalhaCache::ArquivoDoCacheInacessivel(std::io::ErrorKind::NotADirectory)
    );
    assert_eq!(fs::read(&ancestral).unwrap(), b"sentinela");
    assert_eq!(fs::read(&parcial).unwrap(), pdf);
    fs::remove_dir_all(raiz).unwrap();
}

// O erro tipado da gravação privada já existe; o cache não deve
// substituí-lo por `Gravacao` quando o diretório é inseguro.
#[cfg(unix)]
#[test]
fn cache_preserva_causa_da_gravacao_privada() {
    use std::os::unix::fs::PermissionsExt;
    let raiz =
        std::env::temp_dir().join(format!("ies-cache-causa-permissoes-{}", std::process::id()));
    fs::create_dir(&raiz).unwrap();
    fs::set_permissions(&raiz, fs::Permissions::from_mode(0o770)).unwrap();
    let causa = gravar_em(&raiz, "sei.exemplo", "001", None, "html", b"teste").unwrap_err();
    assert_eq!(
        causa,
        FalhaCache::GravacaoPrivadaFalhou(FalhaDaGravacao::DiretorioGravavelPorOutros)
    );
    fs::set_permissions(&raiz, fs::Permissions::from_mode(0o700)).unwrap();
    fs::remove_dir_all(raiz).unwrap();
}

#[test]
fn conteudo_assinado_nao_e_persistido() {
    let home = std::env::temp_dir().join(format!("ies-cache-seguranca-{}", std::process::id()));
    assert_eq!(
        gravar_em(&home, "sei.exemplo", "1", None, "html", b"infra_hash=x"),
        Err(FalhaCache::ConteudoInseguro)
    );
    assert!(!home.exists());
}
