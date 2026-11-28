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
    // O consumidor de #875 também precisa da sigla, não só do ID.
    // Oráculo independente: Todos @caabe699, test_obter_pelo_id_ou_pelo_nome_traz_as_unidades.
    assert_eq!(grupo.unidades[0].nome, "TESTE_1_1 - Unidade de Testes 1.1");
    let alterado = ler_grupo(&tela("grupo_unidade_consultar", 2), "6").unwrap();
    assert_eq!(
        alterado
            .unidades
            .iter()
            .map(|u| u.id.as_str())
            .collect::<Vec<_>>(),
        ["110000002", "110000003"]
    );
    assert_eq!(
        alterado
            .unidades
            .iter()
            .map(|u| u.nome.split_once(" - ").map(|(sigla, _)| sigla))
            .collect::<Vec<_>>(),
        [Some("TESTE_1_1"), Some("TESTE_1_2")]
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

mod fluxo {
    use super::*;
    use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
    use crate::transporte::{FalhaTransporte, RespostaWeb};
    use std::collections::VecDeque;
    use std::io::Cursor;

    const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
    const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const A: &str = "<a id='lnkInfraUnidade' href='#'>UNIDADE A</a>";
    const B: &str = "<a id='lnkInfraUnidade' href='#'>UNIDADE B</a>";

    fn controle() -> String {
        format!(
            "<html><body>{A}<a href='controlador.php?acao=grupo_unidade_listar&amp;infra_hash={HASH}'>Grupos</a>\
             <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
             <table id='tblProcessosGerados'><caption>0 registros:</caption></table></body></html>"
        )
    }

    fn lista(barra: &str, indice: usize) -> String {
        let pos = indice + 1;
        format!(
            "<html><body>{barra}<form id='frmGrupoUnidadeLista' method='post' \
             action='controlador.php?acao=grupo_unidade_listar&amp;infra_hash={HASH}'>\
             <input type='hidden' name='hdnInfraPaginaAtual' value='{indice}'>\
             <select id='selInfraPaginacaoSuperior' name='selInfraPaginacaoSuperior'>\
             <option value='0'>1</option><option value='1'>2</option></select>\
             <table><caption>Grupos de Envio (2 registros - {pos} a {pos}):</caption>\
             <tr><th></th><th>Nome</th><th>Acoes</th></tr>\
             <tr><td><input type='checkbox' value='6'></td><td>Grupo</td>\
             <td><a href='controlador.php?acao=grupo_unidade_consultar&amp;id_grupo_unidade=6&amp;infra_hash={HASH}'>Consultar</a></td></tr>\
             </table></form></body></html>"
        )
    }

    struct Envio {
        respostas: VecDeque<String>,
        registro: Vec<&'static str>,
    }

    impl Envio {
        fn proxima(&mut self) -> RespostaWeb<Cursor<Vec<u8>>> {
            let corpo = self.respostas.pop_front().expect("requisição inesperada");
            crate::transporte::testes_http::resposta(
                200,
                "text/html; charset=utf-8",
                corpo.into_bytes(),
                None,
            )
        }
    }

    impl EnvioDoControle for Envio {
        type Corpo = Cursor<Vec<u8>>;

        fn get(&mut self, _: &str, _: &str) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
            self.registro.push("GET");
            Ok(self.proxima())
        }

        fn post(
            &mut self,
            submissao: SubmissaoDerivada,
            _: &str,
        ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
            assert!(submissao.url().contains("acao=grupo_unidade_listar"));
            self.registro.push("POST");
            Ok(RespostaDoPost {
                resposta: self.proxima(),
                localizacao: None,
            })
        }
    }

    #[test]
    fn fluxo_confere_unidade_no_get_e_post_sem_retry() {
        let pasta = std::env::temp_dir().join(format!("ies-grupo-unidade-{}", std::process::id()));
        std::fs::create_dir_all(&pasta).unwrap();
        let url = pasta.join("url");
        let origem = pasta.join("origem");
        let cookie = pasta.join("cookie");
        for (arquivo, valor) in [
            (
                &url,
                format!("{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_hash={HASH}"),
            ),
            (&origem, ORIGEM.to_owned()),
            (&cookie, "PHPSESSID=SINTETICO".to_owned()),
        ] {
            std::fs::write(arquivo, valor).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(arquivo, std::fs::Permissions::from_mode(0o600)).unwrap();
            }
        }
        let sessao = ReferenciasDaSessao::ler(&url, &origem, &cookie).unwrap();
        let _ = std::fs::remove_dir_all(&pasta);

        // A divergência no GET deve vencer antes de qualquer POST.
        let mut envio = Envio {
            respostas: [controle(), lista(B, 0)].into(),
            registro: Vec::new(),
        };
        let erro =
            ler_via(&sessao, None, &Acao::Listar { pagina: Some(2) }, &mut envio).unwrap_err();
        assert_eq!(
            erro.downcast_ref::<ErroGrupo>(),
            Some(&ErroGrupo::UnidadeInesperada)
        );
        assert_eq!(envio.registro, ["GET", "GET"]);

        for (barra, esperada) in [(A, true), ("", true), (B, false)] {
            let mut envio = Envio {
                respostas: [controle(), lista(barra, 0)].into(),
                registro: Vec::new(),
            };
            let resultado = ler_via(&sessao, None, &Acao::Listar { pagina: None }, &mut envio);
            assert_eq!(resultado.is_ok(), esperada, "{resultado:?}");
            assert_eq!(envio.registro, ["GET", "GET"]);
            if !esperada {
                assert_eq!(
                    resultado.unwrap_err().downcast_ref::<ErroGrupo>(),
                    Some(&ErroGrupo::UnidadeInesperada)
                );
            }
        }
        for (barra, esperada) in [(A, true), ("", true), (B, false)] {
            let mut envio = Envio {
                respostas: [controle(), lista(A, 0), lista(barra, 1)].into(),
                registro: Vec::new(),
            };
            let resultado = ler_via(&sessao, None, &Acao::Listar { pagina: Some(2) }, &mut envio);
            assert_eq!(resultado.is_ok(), esperada, "{resultado:?}");
            assert_eq!(envio.registro, ["GET", "GET", "POST"]);
            if !esperada {
                assert_eq!(
                    resultado.unwrap_err().downcast_ref::<ErroGrupo>(),
                    Some(&ErroGrupo::UnidadeInesperada)
                );
            }
        }
        for (barra, esperada) in [(A, true), ("", true), (B, false)] {
            let consulta = format!("<html><body>{barra}{GRUPO}</body></html>");
            let mut envio = Envio {
                respostas: [controle(), lista(A, 0), consulta].into(),
                registro: Vec::new(),
            };
            let resultado = ler_via(
                &sessao,
                None,
                &Acao::Obter { id: "6".to_owned() },
                &mut envio,
            );
            assert_eq!(resultado.is_ok(), esperada, "{resultado:?}");
            assert_eq!(envio.registro, ["GET", "GET", "GET"]);
            if !esperada {
                assert_eq!(
                    resultado.unwrap_err().downcast_ref::<ErroGrupo>(),
                    Some(&ErroGrupo::UnidadeInesperada)
                );
            }
        }
    }
}
