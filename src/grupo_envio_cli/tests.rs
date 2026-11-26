use super::*;

const GRUPO: &str = "<form id='frmGrupoCadastro'><input name='hdnIdGrupoUnidade' value='6'><input name='txtNome' value='Grupo'><textarea name='txaDescricao'>Descrição</textarea><input name='hdnUnidades' value=''><select name='selUnidades'><option value='11'>Mesmo nome</option><option value='12'>Mesmo nome</option></select></form>";

#[test]
fn consulta_preserva_identidade_dos_membros_homonimos() {
    let grupo = ler_grupo(&HtmlServido::de_teste(GRUPO), "6").unwrap();
    assert_eq!(
        grupo.unidades.into_iter().map(|u| u.id).collect::<Vec<_>>(),
        ["11", "12"]
    );
}

#[test]
fn consulta_tipificada_preserva_contrato_json_e_ids_opacos() {
    let html = GRUPO.replace("value='11'", "value='0011'");
    let grupo = ler_grupo(&HtmlServido::de_teste(&html), "6").unwrap();
    assert_eq!(grupo.unidades[0].id, "0011");
    assert_eq!(grupo.unidades[1].id, "12");
    assert_eq!(
        serde_json::to_value(&grupo).unwrap(),
        serde_json::json!({
            "id": "6",
            "nome": "Grupo",
            "descricao": "Descrição",
            "unidades": [
                {"id": "0011", "nome": "Mesmo nome"},
                {"id": "12", "nome": "Mesmo nome"}
            ]
        })
    );
}

#[test]
fn consulta_recusa_id_trocado_e_membro_duplicado() {
    assert_eq!(
        ler_grupo(&HtmlServido::de_teste(GRUPO), "7"),
        Err(ErroGrupo::ConsultaInesperada)
    );
    let repetido = GRUPO.replace("value='12'", "value='11'");
    assert_eq!(
        ler_grupo(&HtmlServido::de_teste(&repetido), "6"),
        Err(ErroGrupo::IdentidadeAmbigua)
    );
}

#[test]
fn lista_vazia_exige_evidencia_no_formulario_proprio() {
    let aviso = "<form id='frmGrupoUnidadeLista'><div id='divInfraAreaTabela'><label>Nenhum registro encontrado.</label></div></form>";
    let contador = "<form id='frmGrupoUnidadeLista'><input name='hdnInfraNroItens' value='0'><div id='divInfraAreaTabela'></div></form>";
    for html in [aviso, contador] {
        assert!(ler_lista(&HtmlServido::de_teste(html)).unwrap().is_empty());
    }

    for html in [
        aviso.replace("frmGrupoUnidadeLista", "outro"),
        aviso.replace("Nenhum registro encontrado.", "Acesso negado"),
        aviso.replace("</label>", "</label><table></table>"),
        aviso.replace(
            "</form>",
            "<select id='selInfraPaginacaoSuperior'></select></form>",
        ),
        aviso.replace(
            "</form>",
            "<input name='hdnInfraNroItens' value='1'></form>",
        ),
        contador.replace("value='0'", "value='2'"),
        contador.replace(
            "value='0'",
            "value='0'><input name='hdnInfraNroItens' value='0'",
        ),
        contador.replace("<div id='divInfraAreaTabela'></div>", ""),
    ] {
        assert!(ler_lista(&HtmlServido::de_teste(&html)).is_err());
    }
}

#[test]
fn lista_preserva_homonimos_e_recusa_contagem_incompleta() {
    let html = "<form id='frmGrupoUnidadeLista'><table><caption>Grupos de Envio (2 registros):</caption><tr><th></th><th>Nome</th></tr><tr><td><input type='checkbox' value='4'></td><td>Mesmo</td></tr><tr><td><input type='checkbox' value='6'></td><td>Mesmo</td></tr></table></form>";
    let linhas = ler_lista(&HtmlServido::de_teste(html)).unwrap();
    assert_eq!(
        linhas.into_iter().map(|l| l.item.id).collect::<Vec<_>>(),
        ["4", "6"]
    );
    let incompleta = html.replace("2 registros", "3 registros");
    assert!(matches!(
        ler_lista(&HtmlServido::de_teste(&incompleta)),
        Err(ErroGrupo::ListaIncompleta)
    ));
}

#[test]
#[ignore = "exige ZIP original PEN1 verificado em IES_ACERVO_CACHE"]
fn cassette_pen1_grupos_envio_preservam_ids() {
    use crate::cassette::acervo::gravacao;
    use crate::cassette::{ZipDoAcervo, conferir_cassette_ate, html_da_resposta};
    use std::path::Path;

    let cache = std::env::var_os("IES_ACERVO_CACHE").expect("IES_ACERVO_CACHE");
    let zip = ZipDoAcervo::ler(&Path::new(&cache).join("todos-sei-cassettes-pen1.zip")).unwrap();
    let card = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cassettes/pen1/grupo-envio-ciclo-pen1--cf5fac874b66.md");
    let gravada = conferir_cassette_ate(&card, &zip, 120_000_000).unwrap();

    // Ordinais e oráculo independente: Todos @caabe699 tests/test_grupo_envio_010.py.
    let tela = |acao: &str, ordem: usize| {
        let r = gravacao::respostas(&gravada.json)
            .into_iter()
            .filter(|r| r.operacao == acao && r.metodo == "GET")
            .nth(ordem)
            .expect("resposta esperada no original");
        assert_eq!(r.status, 200);
        HtmlServido::de_teste(&html_da_resposta(r.interacao).expect("HTML da cassette"))
    };
    let lista = ler_lista(&tela("grupo_unidade_listar", 2)).unwrap();
    let ids: Vec<_> = lista.iter().map(|l| l.item.id.as_str()).collect();
    assert_eq!(ids, ["4", "6"]);
    let grupo = ler_grupo(&tela("grupo_unidade_consultar", 0), "6").unwrap();
    assert_eq!(
        grupo
            .unidades
            .iter()
            .map(|u| u.id.as_str())
            .collect::<Vec<_>>(),
        ["110000002"]
    );
    let alterado = ler_grupo(&tela("grupo_unidade_consultar", 2), "6").unwrap();
    assert_eq!(
        alterado
            .unidades
            .iter()
            .map(|u| u.id.as_str())
            .collect::<Vec<_>>(),
        ["110000002", "110000003"]
    );
    let antes = ler_lista(&tela("grupo_unidade_listar", 0)).unwrap();
    assert_eq!(
        antes.iter().map(|l| l.item.id.as_str()).collect::<Vec<_>>(),
        ["4"]
    );
    let depois = ler_lista(&tela("grupo_unidade_listar", 13)).unwrap();
    // A exclusão do grupo 6 não deve apagar nem modificar o grupo preexistente.
    assert_eq!(
        depois.iter().map(|l| &l.item).collect::<Vec<_>>(),
        antes.iter().map(|l| &l.item).collect::<Vec<_>>()
    );
}

#[test]
fn unidade_publicada_distingue_ausencia_de_barra_invalida() {
    const A: &str = "<a id='lnkInfraUnidade' href='#'>UNIDADE A</a>";
    const B: &str = "<a id='lnkInfraUnidade' href='#'>UNIDADE B</a>";
    let par = format!(
        "<div id='divInfraBarraSistemaMovel'>{A}<div id='divInfraBarraSistemaPadrao'>{A}</div></div>"
    );
    for html in ["".to_owned(), A.to_owned(), par.clone()] {
        assert!(conferir_unidade_publicada(&HtmlServido::de_teste(&html), "UNIDADE A").is_ok());
    }
    for html in [
        B.to_owned(),
        "<a id='lnkInfraUnidade'> </a>".to_owned(),
        "<span id='lnkInfraUnidade'>UNIDADE A</span>".to_owned(),
        format!("{A}{A}"),
        format!("{par}{B}"),
        par.replacen("UNIDADE A", "UNIDADE B", 1),
    ] {
        assert!(conferir_unidade_publicada(&HtmlServido::de_teste(&html), "UNIDADE A").is_err());
    }
}
