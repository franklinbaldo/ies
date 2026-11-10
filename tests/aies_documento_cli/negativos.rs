use super::*;

#[test]
fn bdd_88_03_cli_nao_inventa_versao_ausente_do_historico() {
    let pasta = Pasta::nova();
    // O documento tem apenas versões 1 e 3. A CLI deve recusar a
    // versão 2 como não observada, sem declarar que nunca existiu.
    // Controle, pesquisa, tela, árvore e histórico: cinco pedidos.
    let saida = executar(
        &pasta,
        &["--json", "documento", DOC, "--versao", "2", "--sem-cache"],
        5,
    );
    assert_eq!(saida.status.code(), Some(1));
    let valor = json(&saida);
    assert_eq!(valor["documento"], DOC);
    assert!(!pasta.0.join("0011519.v2.html").exists());
}

/// BDD-88-03: uma sessão funcional não concede acesso a um DOC ausente
/// do estado compartilhado; a recusa vem da pesquisa HTTP do simulador.
#[test]
fn bdd_88_03_cli_documento_nao_acessivel_na_sessao() {
    let pasta = Pasta::nova();
    let destino = pasta.0.join("saida");
    fs::create_dir(&destino).unwrap();
    let args = [
        "--json",
        "documento",
        DOC,
        "--versao",
        "1",
        "--saida",
        destino.to_str().unwrap(),
        "--sem-cache",
    ];
    // Controle conhecido, busca sem snapshot de DOC: um GET e um POST
    // local de leitura. O simulador recusa com 503, sem inventar ausência.
    let saida = executar_com_estado(&pasta, &args, 2, false);
    assert_ne!(
        saida.status.code(),
        Some(0),
        "documento ausente virou sucesso"
    );
    let valor = json(&saida);
    assert!(
        valor["documentos"].is_null(),
        "publicou documento não observado: {valor}"
    );
    assert!(!destino.join("0011519.v1.html").exists());
}

#[test]
fn bdd_88_03_cli_recusa_versao_sem_acesso_na_sessao() {
    let pasta = Pasta::nova();
    let destino = pasta.0.join("saida");
    fs::create_dir(&destino).unwrap();
    let args = [
        "--json",
        "documento",
        DOC,
        "--versao",
        "1",
        "--saida",
        destino.to_str().unwrap(),
        "--sem-cache",
    ];
    // Controle, pesquisa, tela, árvore, lista e versão restrita:
    // nenhum POST modificador e nenhuma foto da versão não acessível.
    let saida = executar_com_restricao(&pasta, &args, 6, true, true);
    assert_eq!(saida.status.code(), Some(1), "não recusou versão restrita");
    let valor = json(&saida);
    assert_eq!(valor["status_http"], 403, "recusa sem HTTP 403: {valor}");
    assert_eq!(valor["documento"], DOC);
    assert!(valor["documentos"].is_null());
    assert!(!destino.join("0011519.v1.html").exists());

    let humano = executar_com_restricao(
        &pasta,
        &["documento", DOC, "--versao", "1", "--sem-cache"],
        6,
        true,
        true,
    );
    assert_eq!(humano.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&humano.stderr).contains("HTTP 403"),
        "recusa humana sem HTTP 403"
    );
}

/// Controle positivo do BDD-88-03: negar a versão 1 não impede
/// a leitura da versão 3 na mesma identidade de documento e cenário.
#[test]
fn bdd_88_03_cli_restricao_de_versao_nao_bloqueia_outras_versoes() {
    let pasta = Pasta::nova();
    let destino = pasta.0.join("saida");
    fs::create_dir(&destino).unwrap();
    let args = [
        "--json",
        "documento",
        DOC,
        "--versao",
        "3",
        "--saida",
        destino.to_str().unwrap(),
        "--sem-cache",
    ];
    let saida = executar_com_restricao(&pasta, &args, 6, true, true);
    assert_eq!(
        saida.status.code(),
        Some(0),
        "bloqueio da versão 1 contaminou versão 3: {}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor = json(&saida);
    assert_eq!(valor["documentos"][0]["numero"], DOC);
    assert_eq!(valor["documentos"][0]["versao"], 3);
    assert_eq!(valor["documentos"][0]["arquivo"]["gravado"], true);
    assert_eq!(valor["requisicoes"], 6);
    let foto = destino.join("0011519.v3.html");
    let html = fs::read_to_string(&foto).unwrap();
    assert!(html.contains("Terceira redação"));
    assert!(!html.contains("Primeira redação"));
    assert!(!destino.join("0011519.v1.html").exists());
}

#[test]
fn bdd_88_03_cli_recusa_sessao_sem_acesso_antes_da_pesquisa() {
    let pasta = Pasta::nova();
    let destino = pasta.0.join("saida");
    fs::create_dir(&destino).unwrap();
    let args = [
        "--json",
        "documento",
        DOC,
        "--versao",
        "1",
        "--saida",
        destino.to_str().unwrap(),
        "--sem-cache",
    ];
    // O primeiro GET de Controle responde pela sessão inacessível;
    // não há pesquisa nem leitura do histórico, muito menos foto.
    let saida = executar_com_sessao(&pasta, &args, 1, true, false, true);
    assert_eq!(saida.status.code(), Some(1));
    let valor = json(&saida);
    assert_eq!(valor["status_http"], 403, "sessão não recusada: {valor}");
    assert!(valor["documentos"].is_null());
    assert!(!destino.join("0011519.v1.html").exists());
}
