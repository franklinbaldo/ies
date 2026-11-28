use super::*;

#[test]
fn previa_le_grupo_servido_sem_confundir_vazio_e_ausencia() {
    let casos = [
        (
            "<select name='selGrupoBloco'><option value='null'>Nenhum</option></select>",
            Some("null"),
        ),
        (
            "<select name='selGrupoBloco'><option value='null'>Nenhum</option><option selected value=''>Todos</option></select>",
            Some(""),
        ),
        (
            "<select name='selGrupoBloco'><option value='null'>Nenhum</option><option selected value='4'>Grupo</option></select>",
            Some("4"),
        ),
        ("<textarea name='txtDescricao'></textarea>", None),
    ];
    for (controles, esperado) in casos {
        let html =
            HtmlServido::de_teste(&format!("<form id='frmBlocoCadastro'>{controles}</form>"));
        assert_eq!(grupo_padrao(&html).unwrap().as_deref(), esperado);
    }
}

#[test]
fn derivacao_preserva_campo_da_versao_grupo_oculto_e_charset() {
    use crate::formulario_web::CodificacaoDaTela;

    let origem = "https://sei.exemplo.gov.br/sei";
    let instalacao = Instalacao::interpretar(origem).unwrap();
    let hash = "a".repeat(64);
    let action = format!("controlador.php?acao=bloco_assinatura_cadastrar&infra_hash={hash}");
    let url = format!("{origem}/{action}");
    let casos = [
        (
            "txtDescricao",
            CodificacaoDaTela::Iso88591,
            "A%E7%E3o+%26+B",
        ),
        (
            "txaDescricao",
            CodificacaoDaTela::Utf8,
            "A%C3%A7%C3%A3o+%26+B",
        ),
    ];
    for (campo, codificacao, codificado) in casos {
        let html = HtmlServido::de_teste(&format!(
            "<form id='frmBlocoCadastro' method='post' action='{action}'>\
            <input type='hidden' name='hdnExemplo' value='original'>\
            <textarea name='{campo}'></textarea>\
            <select name='selGrupoBloco'><option value='null'>Nenhum</option></select>\
            <button type='submit' name='sbmCadastrarBloco' value='Salvar'>Salvar</button></form>"
        ));
        let tela = TelaServida {
            url: &url,
            html: &html,
            codificacao,
        };
        let submissao = derivar_cadastro(&instalacao, &tela, "TESTE", "Ação & B", "null").unwrap();
        assert_eq!(
            std::str::from_utf8(submissao.corpo()).unwrap(),
            format!(
                "hdnExemplo=original&{campo}={codificado}&selGrupoBloco=null&sbmCadastrarBloco=Salvar"
            )
        );
        assert_eq!(submissao.url(), url);
    }
}

#[test]
fn aceite_publica_identidade_unica_na_lista_da_mesma_origem() {
    let url = "https://sei.exemplo.gov.br/sei/controlador.php?acao=bloco_assinatura_cadastrar";
    assert_eq!(
        id_do_aceite(
            "controlador.php?acao=bloco_assinatura_listar&id_bloco=42",
            url
        ),
        Some("42")
    );
    for local in [
        "controlador.php?acao=bloco_assinatura_listar",
        "controlador.php?acao=bloco_assinatura_listar&id_bloco=42&id_bloco=43",
        "controlador.php?acao=bloco_assinatura_listar&id_bloco=x",
        "controlador.php?acao=bloco_interno_listar&id_bloco=42",
        "https://outro.example/sei/controlador.php?acao=bloco_assinatura_listar&id_bloco=42",
    ] {
        assert!(id_do_aceite(local, url).is_none());
    }
}

#[test]
fn releitura_exige_id_novo_estado_e_descricao() {
    let linha = |numero: &str, estado, descricao: &str| bloco::BlocoLido {
        numero: numero.to_owned(),
        tipo: TipoBloco::Assinatura,
        estado,
        descricao: descricao.to_owned(),
    };
    let lista = |itens: Vec<bloco::BlocoLido>| bloco::ListaDeBlocos {
        tipo: TipoBloco::Assinatura,
        total_declarado: itens.len(),
        faixa_observada: None,
        itens,
    };
    let antiga = linha("41", EstadoDoBloco::Gerado, "Mesmo texto");
    let antes = lista(vec![antiga.clone()]);
    let nova = linha("42", EstadoDoBloco::Gerado, "Mesmo texto");
    let depois = lista(vec![antiga.clone(), nova.clone()]);
    assert_eq!(
        conferir_criacao(&antes, &depois, "42", "Mesmo  texto"),
        Some(&nova)
    );
    // Cadastro vazio observado também é completo (0/0).
    let vazio = lista(Vec::new());
    assert_eq!(
        conferir_criacao(&vazio, &depois, "42", "Mesmo texto"),
        Some(&nova)
    );

    // A releitura só precisa mostrar a identidade procurada; não
    // requer que todos os demais blocos estejam na mesma página.
    let mut posterior_parcial = lista(vec![nova.clone()]);
    posterior_parcial.total_declarado = 2;
    posterior_parcial.faixa_observada = Some(crate::legenda::Faixa { inicio: 2, fim: 2 });
    assert_eq!(
        conferir_criacao(&antes, &posterior_parcial, "42", "Mesmo texto"),
        Some(&nova)
    );

    assert!(conferir_criacao(&antes, &depois, "41", "Mesmo texto").is_none());
    assert!(conferir_criacao(&antes, &depois, "43", "Mesmo texto").is_none());
    assert!(conferir_criacao(&antes, &depois, "42", "Outro texto").is_none());
    let errado = lista(vec![linha("42", EstadoDoBloco::Recebido, "Mesmo texto")]);
    assert!(conferir_criacao(&antes, &errado, "42", "Mesmo texto").is_none());
}

#[test]
fn novo_duplicado_so_quando_mesmo_destino() {
    let botao = "<button id='btnNovo' onclick=\"location.href='controlador.php?acao=bloco_assinatura_cadastrar'\">Novo</button>";
    let html = HtmlServido::de_teste(&format!("{botao}{botao}"));
    assert_eq!(
        link_novo(&html).unwrap(),
        "controlador.php?acao=bloco_assinatura_cadastrar"
    );
    let outro = botao.replace("cadastrar", "listar");
    let html = HtmlServido::de_teste(&format!("{botao}{outro}"));
    assert!(link_novo(&html).is_err());
    assert!(link_novo(&HtmlServido::de_teste("<html></html>")).is_err());
}
