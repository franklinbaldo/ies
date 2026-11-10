use super::*;

#[test]
fn bdd_89_02_cli_publica_uma_versao_e_confere_por_releitura() {
    let pasta = Pasta::nova();
    let leitura = executar(
        &pasta,
        &[
            "--json",
            "documento",
            DOC,
            "--versao",
            "3",
            "--saida",
            pasta.0.to_str().unwrap(),
            "--sem-cache",
        ],
        6,
    );
    assert_eq!(leitura.status.code(), Some(0), "{:?}", json(&leitura));
    let base = fs::read_to_string(pasta.0.join("0011519.v3.html")).unwrap();
    assert!(base.contains("Terceira redação"));
    let editada = base.replace("<p>Terceira redação</p>", "");
    assert_ne!(base, editada);
    pasta.privado("0011519.v4.html", &editada);
    let resposta = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "salvar",
            DOC,
            "--versao",
            "3",
            "--confirmar",
        ],
        10, // seis leituras + editor + POST + versão e assinaturas relidas.
    );
    assert_eq!(
        resposta.status.code(),
        Some(0),
        "falha: {:?}; stderr={}",
        json(&resposta),
        String::from_utf8_lossy(&resposta.stderr),
    );
    let salvo = json(&resposta);
    assert_eq!(salvo["confirmado"], true);
    assert_eq!(salvo["versao_criada"], 4);
    assert_eq!(salvo["releitura_confirmada"], true);
    assert_eq!(
        fs::read_to_string(pasta.0.join("0011519.v3.html")).unwrap(),
        base
    );
    assert_eq!(
        fs::read_to_string(pasta.0.join("0011519.v4.html")).unwrap(),
        editada
    );
}

#[test]
fn bdd_89_03_cli_recusa_foto_local_divergente_sem_escrever() {
    let pasta = Pasta::nova();
    let leitura = executar(
        &pasta,
        &[
            "--json",
            "documento",
            DOC,
            "--versao",
            "3",
            "--saida",
            pasta.0.to_str().unwrap(),
            "--sem-cache",
        ],
        6,
    );
    assert_eq!(leitura.status.code(), Some(0), "{:?}", json(&leitura));
    let base = fs::read_to_string(pasta.0.join("0011519.v3.html")).unwrap();
    let divergente = base.replace("Terceira redação", "Base local modificada");
    pasta.privado("0011519.v3.html", &divergente);
    pasta.privado(
        "0011519.v4.html",
        &divergente.replace("modificada", "editada"),
    );
    let resposta = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "salvar",
            DOC,
            "--versao",
            "3",
            "--confirmar",
        ],
        6, // conferência da leitura, sem chegar ao editor ou ao POST.
    );
    assert_eq!(resposta.status.code(), Some(1));
    let erro = json(&resposta);
    assert!(erro["erro"].as_str().unwrap().contains("base local"));
    assert_eq!(
        fs::read_to_string(pasta.0.join("0011519.v3.html")).unwrap(),
        divergente,
    );
}

#[test]
fn bdd_89_04_cli_documento_assinado_confirma_estado_apos_edicao() {
    let pasta = Pasta::nova();
    let leitura = executar_com_assinaturas(
        &pasta,
        &[
            "--json",
            "documento",
            DOC,
            "--versao",
            "3",
            "--saida",
            pasta.0.to_str().unwrap(),
            "--sem-cache",
        ],
        6,
        true,
        2,
    );
    assert_eq!(leitura.status.code(), Some(0), "{:?}", json(&leitura));
    let base = fs::read_to_string(pasta.0.join("0011519.v3.html")).unwrap();
    pasta.privado(
        "0011519.v4.html",
        &base.replace("Terceira redação", "Assinatura removida após edição"),
    );
    // A oferta do editor informa duas assinaturas. Após o POST único,
    // a releitura do editor confirma contagem zero, além da versão nova.
    let resposta = executar_com_assinaturas(
        &pasta,
        &[
            "--json",
            "documento",
            "salvar",
            DOC,
            "--versao",
            "3",
            "--confirmar",
        ],
        10,
        true,
        2,
    );
    assert_eq!(
        resposta.status.code(),
        Some(0),
        "falha: {:?}; stderr={}",
        json(&resposta),
        String::from_utf8_lossy(&resposta.stderr),
    );
    let publicado = json(&resposta);
    assert_eq!(publicado["versao_criada"], 4);
    assert_eq!(publicado["assinaturas_apos"], 0);
    assert_eq!(publicado["releitura_confirmada"], true);
    assert_eq!(
        fs::read_to_string(pasta.0.join("0011519.v3.html")).unwrap(),
        base
    );
}

#[test]
fn bdd_89_03_cli_nao_ignora_edicao_na_moldura_do_documento() {
    let pasta = Pasta::nova();
    let leitura = executar(
        &pasta,
        &[
            "--json",
            "documento",
            DOC,
            "--versao",
            "3",
            "--saida",
            pasta.0.to_str().unwrap(),
            "--sem-cache",
        ],
        6,
    );
    assert_eq!(leitura.status.code(), Some(0));
    let base = fs::read_to_string(pasta.0.join("0011519.v3.html")).unwrap();
    let edicao = base
        .replace("Terceira redação", "Quarta redação")
        .replace("<title>", "<title>Alterado ");
    pasta.privado("0011519.v4.html", &edicao);
    let resposta = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "salvar",
            DOC,
            "--versao",
            "3",
            "--confirmar",
        ],
        6, // Leitura remota; nem consulta ao editor nem POST.
    );
    assert_eq!(resposta.status.code(), Some(1));
    assert!(
        json(&resposta)["erro"]
            .as_str()
            .unwrap()
            .contains("região não editável")
    );
    assert_eq!(
        fs::read_to_string(pasta.0.join("0011519.v3.html")).unwrap(),
        base
    );
}

#[test]
fn bdd_89_02_cli_consulta_posterior_observa_versao_publicada() {
    let pasta = Pasta::nova();
    let leitura = executar(
        &pasta,
        &[
            "--json",
            "documento",
            DOC,
            "--versao",
            "3",
            "--saida",
            pasta.0.to_str().unwrap(),
            "--sem-cache",
        ],
        6,
    );
    assert_eq!(leitura.status.code(), Some(0), "{:?}", json(&leitura));
    let base = fs::read_to_string(pasta.0.join("0011519.v3.html")).unwrap();
    let editada = base.replace("Terceira redação", "Texto publicado na quarta versão");
    pasta.privado("0011519.v4.html", &editada);
    let salvo = executar(
        &pasta,
        &[
            "--json",
            "documento",
            "salvar",
            DOC,
            "--versao",
            "3",
            "--confirmar",
        ],
        10,
    );
    assert_eq!(salvo.status.code(), Some(0), "{:?}", json(&salvo));
    assert_eq!(json(&salvo)["versao_criada"], 4);

    // Novo processo CLI, nova conexão ao mesmo SQLite efêmero do ensaio.
    // A leitura posterior deve encontrar o efeito persistido, sem recriar a fixture.
    let destino = pasta.0.join("depois");
    fs::create_dir(&destino).unwrap();
    let releitura = executar(
        &pasta,
        &[
            "--json",
            "documento",
            DOC,
            "--versao",
            "4",
            "--saida",
            destino.to_str().unwrap(),
            "--sem-cache",
        ],
        6,
    );
    assert_eq!(
        releitura.status.code(),
        Some(0),
        "consulta posterior falhou: stderr={}; stdout={}",
        String::from_utf8_lossy(&releitura.stderr),
        String::from_utf8_lossy(&releitura.stdout)
    );
    assert_eq!(json(&releitura)["documentos"][0]["versao"], 4);
    let publicada = fs::read_to_string(destino.join("0011519.v4.html")).unwrap();
    assert!(publicada.contains("Texto publicado na quarta versão"));
    assert!(!publicada.contains("Terceira redação"));
}
