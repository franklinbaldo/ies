//! Testes sintéticos do fluxo Web de unidades (issue #244).
use super::*;
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::RespostaDoPost;
use crate::tela_html::HtmlServido;
use crate::transporte::RespostaWeb;
use std::fmt::Write as _;
use std::io::Cursor;
const ORIGEM: &str = "https://sei.example/sei";
const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn url(acao: AcaoDoSei) -> String {
    format!(
        "{ORIGEM}/controlador.php?acao={acao}&infra_hash={HASH}",
        acao = acao.como_na_url()
    )
}
fn resposta(html: String) -> RespostaDoPost<Cursor<Vec<u8>>> {
    RespostaDoPost {
        resposta: RespostaWeb {
            status_code: 200,
            mime: "text/html; charset=utf-8".to_owned(),
            content_length: None,
            localizacao: None,
            corpo: Cursor::new(html.into_bytes()),
        },
        localizacao: None,
    }
}
fn pagina(p: usize, ids: &[&str]) -> String {
    let linhas: String = ids
        .iter()
        .map(|id| {
            [
                "<tr><td><input type='checkbox' value='",
                id,
                "'></td><td>ABC/",
                id,
                "</td><td>Descrição</td><td>Órgão</td></tr>",
            ]
            .concat()
        })
        .collect();
    let acao = url(AcaoDoSei::UnidadeSelecionarEnvioProcesso);
    format!(
        "<form id='frmUnidadeLista' method='post' action='{acao}'>
         <input type='hidden' name='hdnInfraPaginaAtual' value='{p}'>
         <select name='selOrgaoUnidade'><option value=''>Todos</option></select>
         <input name='txtSiglaUnidade' value=''>
         <input name='txtDescricaoUnidade' value=''>
         <select name='selInfraPaginacaoSuperior'>
           <option value='0'>1</option><option value='1'>2</option>
         </select>
         <select name='selInfraPaginacaoInferior'>
           <option value='0'>1</option><option value='1'>2</option>
         </select>
         <input type='submit' name='btnPesquisar' value='Pesquisar'>
         <table class='infraTable'><caption>Lista de Unidades (3 registros):</caption>
         <tr><th></th><th>Sigla</th><th>Descrição</th><th>Órgão</th></tr>
         {linhas}</table></form>"
    )
}
struct Fake {
    gets: usize,
    posts: usize,
    repetir: bool,
    filtro_sobreposto: bool,
}
impl TransporteLupa for Fake {
    type Corpo = Cursor<Vec<u8>>;
    fn get(
        &mut self,
        alvo: &str,
        _cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        let passo = self.gets;
        self.gets += 1;
        let (acao, html) = match passo {
            0 => (
                "procedimento_controlar",
                format!(
                    "<a href='{}'>Grupos</a>",
                    url(AcaoDoSei::GrupoUnidadeListar)
                ),
            ),
            1 => (
                "grupo_unidade_listar",
                format!(
                    "<button id='btnNovo' onclick=\"window.location.href='{}';\">Novo</button>",
                    url(AcaoDoSei::GrupoUnidadeCadastrar)
                ),
            ),
            2 => (
                "grupo_unidade_cadastrar",
                format!(
                    "<a href='{}'>Unidades</a>",
                    url(AcaoDoSei::UnidadeSelecionarEnvioProcesso)
                ),
            ),
            3 => ("unidade_selecionar_envio_processo", pagina(0, &[])),
            _ => panic!("GET fora do orçamento"),
        };
        assert!(alvo.contains(&format!("acao={acao}")));
        Ok(resposta(html))
    }
    fn post(
        &mut self,
        envio: SubmissaoDerivada,
        _cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(envio.url(), url(AcaoDoSei::UnidadeSelecionarEnvioProcesso));
        let indice_pagina = self.posts;
        assert!(
            String::from_utf8_lossy(envio.corpo())
                .contains(&format!("hdnInfraPaginaAtual={indice_pagina}"))
        );
        self.posts += 1;
        let ids: &[&str] = if indice_pagina == 0 {
            &["1", "2"]
        } else if self.repetir {
            &["1"]
        } else {
            &["3"]
        };
        let html = pagina(indice_pagina, ids);
        let html = if self.filtro_sobreposto {
            html.replace(
                "name='txtSiglaUnidade' value=''",
                "name='txtSiglaUnidade' value='FILTRADA'",
            )
        } else {
            html
        };
        Ok(resposta(html))
    }
}
#[test]
fn quatro_get_e_um_post_por_pagina_sem_repetir() {
    let mut rede = Fake {
        gets: 0,
        posts: 0,
        repetir: false,
        filtro_sobreposto: false,
    };
    let lida = ler_integral(
        &mut rede,
        &url(AcaoDoSei::ProcedimentoControlar),
        ORIGEM,
        "PHPSESSID=SINTETICO",
    )
    .unwrap();
    assert_eq!(lida.requisicoes, 6);
    assert_eq!(lida.total, 3);
    assert_eq!((rede.gets, rede.posts), (4, 2));
}
#[test]
fn filtro_restaurado_pelo_sei_nao_vira_lista_integral() {
    // Caso do Todos tests/test_destinos_indice_010.py @323eb01:
    // uma lista filtrada não substitui o snapshot integral.
    let mut rede = Fake {
        gets: 0,
        posts: 0,
        repetir: false,
        filtro_sobreposto: true,
    };
    assert_eq!(
        ler_integral(
            &mut rede,
            &url(AcaoDoSei::ProcedimentoControlar),
            ORIGEM,
            "PHPSESSID=SINTETICO",
        )
        .err(),
        Some(FalhaDaLupa::Tela(ErroLupa::PaginaInvalida)),
    );
    assert_eq!(
        (rede.gets, rede.posts),
        (4, 1),
        "falha fechada sem novo POST"
    );
}
#[test]
fn pagina_repetida_recusa_antes_de_persistir() {
    let mut rede = Fake {
        gets: 0,
        posts: 0,
        repetir: true,
        filtro_sobreposto: false,
    };
    let err = ler_integral(
        &mut rede,
        &url(AcaoDoSei::ProcedimentoControlar),
        ORIGEM,
        "PHPSESSID=SINTETICO",
    )
    .err()
    .unwrap();
    assert_eq!(err, FalhaDaLupa::Tela(ErroLupa::PaginasDivergentes));
    assert_eq!(rede.posts, 2);
}

// O primeiro GET real da cassette GO anuncia 13.671 unidades em 274
// paginas. O acervo contem somente seis respostas; a regressao abaixo
// gera paginas SINTETICAS para exercitar todo o ciclo, sem SEI nem
// credenciais. Caso de Todos tests/test_lupa_unidades_010.py @323eb01.
const TOTAL_GO_SINTETICO: usize = 13_671;
const TAMANHO_PAGINA: usize = 50;
const PAGINAS_GO_SINTETICAS: usize = 274;

fn pagina_integral_sintetica(indice: usize, adulterar_filtro: bool) -> String {
    let inicio = indice * TAMANHO_PAGINA;
    let fim = ((indice + 1) * TAMANHO_PAGINA).min(TOTAL_GO_SINTETICO);
    let ids: Vec<String> = (inicio..fim).map(|id| id.to_string()).collect();
    let referencias: Vec<&str> = ids.iter().map(String::as_str).collect();
    let mut opcoes = String::new();
    for i in 0..PAGINAS_GO_SINTETICAS {
        write!(&mut opcoes, "<option value='{i}'>{}</option>", i + 1)
            .expect("gravar string sintetica");
    }
    let html = pagina(indice, &referencias)
        .replace(
            "<option value='0'>1</option><option value='1'>2</option>",
            &opcoes,
        )
        .replace(
            "Lista de Unidades (3 registros):",
            "Lista de Unidades (13671 registros):",
        );
    if adulterar_filtro {
        html.replace(
            "name='txtSiglaUnidade' value=''",
            "name='txtSiglaUnidade' value='PARCIAL'",
        )
    } else {
        html
    }
}

struct RoteiroPaginacaoIntegral {
    navegacao: Fake,
    posts: usize,
    adulterar_ultima: bool,
}

impl RoteiroPaginacaoIntegral {
    fn novo(adulterar_ultima: bool) -> Self {
        Self {
            navegacao: Fake {
                gets: 0,
                posts: 0,
                repetir: false,
                filtro_sobreposto: false,
            },
            posts: 0,
            adulterar_ultima,
        }
    }
}

impl TransporteLupa for RoteiroPaginacaoIntegral {
    type Corpo = Cursor<Vec<u8>>;

    fn get(
        &mut self,
        alvo: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.navegacao.get(alvo, cookie)
    }

    fn post(
        &mut self,
        envio: SubmissaoDerivada,
        _cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(envio.url(), url(AcaoDoSei::UnidadeSelecionarEnvioProcesso));
        let indice = self.posts;
        assert!(
            indice < PAGINAS_GO_SINTETICAS,
            "sem POST extra nem repeticao de pagina"
        );
        assert!(
            String::from_utf8_lossy(envio.corpo())
                .contains(&format!("hdnInfraPaginaAtual={indice}")),
            "POST deve pedir a pagina atual exatamente uma vez"
        );
        self.posts += 1;
        Ok(resposta(pagina_integral_sintetica(
            indice,
            self.adulterar_ultima && indice + 1 == PAGINAS_GO_SINTETICAS,
        )))
    }
}

#[test]
fn pagina_integral_de_274_paginas_nao_pula_nem_repete_requisicoes() {
    let mut roteiro = RoteiroPaginacaoIntegral::novo(false);
    let lida = ler_integral(
        &mut roteiro,
        &url(AcaoDoSei::ProcedimentoControlar),
        ORIGEM,
        "PHPSESSID=SINTETICO",
    )
    .expect("todos os IDs e totais sao consistentes");
    assert_eq!(lida.total, TOTAL_GO_SINTETICO);
    assert_eq!(lida.unidades.len(), TOTAL_GO_SINTETICO);
    assert_eq!(lida.unidades[0].id, "0");
    assert_eq!(lida.unidades[TOTAL_GO_SINTETICO - 1].id, "13670");
    assert_eq!(lida.requisicoes, 4 + PAGINAS_GO_SINTETICAS);
    assert_eq!(roteiro.navegacao.gets, 4);
    assert_eq!(roteiro.posts, PAGINAS_GO_SINTETICAS);
}

#[test]
fn filtro_adulterado_na_ultima_das_274_paginas_aborta_sem_retry() {
    let mut roteiro = RoteiroPaginacaoIntegral::novo(true);
    assert_eq!(
        ler_integral(
            &mut roteiro,
            &url(AcaoDoSei::ProcedimentoControlar),
            ORIGEM,
            "PHPSESSID=SINTETICO",
        )
        .err(),
        Some(FalhaDaLupa::Tela(ErroLupa::PaginaInvalida)),
        "falha fechada: nenhuma lista parcial deve ser devolvida para persistencia"
    );
    assert_eq!(roteiro.navegacao.gets, 4);
    assert_eq!(roteiro.posts, PAGINAS_GO_SINTETICAS);
}

#[test]
fn falha_de_filtro_publica_cinco_tentativas_sem_post_repetido() {
    // Cenario derivado da falha GO: quatro GET de navegacao e uma
    // consulta rejeitada nao podem sumir da telemetria de falha.
    let mut rede = Fake {
        gets: 0,
        posts: 0,
        repetir: false,
        filtro_sobreposto: true,
    };
    let mut requisicoes = 0;
    assert_eq!(
        ler_integral_com_contador(
            &mut rede,
            &url(AcaoDoSei::ProcedimentoControlar),
            ORIGEM,
            "PHPSESSID=SINTETICO",
            &mut requisicoes,
        )
        .err(),
        Some(FalhaDaLupa::Tela(ErroLupa::PaginaInvalida)),
    );
    assert_eq!(requisicoes, 5, "4 GET + 1 POST, mesmo na recusa");
    assert_eq!((rede.gets, rede.posts), (4, 1));
}

#[test]
fn falha_tardia_na_pagina_final_conta_todas_as_tentativas() {
    // A pagina final rejeitada nao apaga as 278 tentativas da varredura.
    let mut roteiro = RoteiroPaginacaoIntegral::novo(true);
    let mut requisicoes = 0;
    assert_eq!(
        ler_integral_com_contador(
            &mut roteiro,
            &url(AcaoDoSei::ProcedimentoControlar),
            ORIGEM,
            "PHPSESSID=SINTETICO",
            &mut requisicoes,
        )
        .err(),
        Some(FalhaDaLupa::Tela(ErroLupa::PaginaInvalida)),
    );
    assert_eq!(requisicoes, 4 + PAGINAS_GO_SINTETICAS);
    assert_eq!((roteiro.navegacao.gets, roteiro.posts), (4, 274));
}

#[test]
fn dois_submits_sem_name_da_lupa_go_nao_entram_no_corpo() {
    // Forma observada no replay GO integral, cassette 85f61d67.
    // Ambos os botoes carregam id, nao name. Browser nao envia
    // pares de botao sem name; os filtros continuam vazios.
    let html = pagina(0, &[]).replace(
        "<input type='submit' name='btnPesquisar' value='Pesquisar'>",
        "<input type='submit' id='btnPesquisar' value='Pesquisar'>\
         <input type='submit' id='btnPesquisar' value='Pesquisar'>",
    );
    let instalacao = Instalacao::interpretar(ORIGEM).expect("origem sintetica");
    let post = consulta(
        &instalacao,
        &url(AcaoDoSei::UnidadeSelecionarEnvioProcesso),
        &HtmlServido::de_teste(&html),
        "text/html; charset=utf-8",
        0,
    )
    .expect("forma GO observada deve derivar uma consulta");
    let corpo = String::from_utf8_lossy(post.corpo());
    assert!(!corpo.contains("btnPesquisar="), "botao sem name nao entra");
    assert!(corpo.contains("hdnInfraPaginaAtual=0"));
    assert!(corpo.contains("selOrgaoUnidade="));
    assert!(corpo.contains("txtSiglaUnidade="));
}

#[test]
fn botoes_sem_nome_ausentes_ou_repetidos_demais_falham_fechados() {
    let instalacao = Instalacao::interpretar(ORIGEM).expect("origem sintetica");
    let forma = pagina(0, &[]).replace("name='btnPesquisar'", "id='btnPesquisar'");
    let ausente = forma.replace("id='btnPesquisar'", "id='outro'");
    let duplicada = forma.replace(
        "</form>",
        "<input type='submit' id='btnPesquisar' value='Pesquisar'>\
         <input type='submit' id='btnPesquisar' value='Pesquisar'></form>",
    );
    for html in [&ausente, &duplicada] {
        assert!(matches!(
            consulta(
                &instalacao,
                &url(AcaoDoSei::UnidadeSelecionarEnvioProcesso),
                &HtmlServido::de_teste(html),
                "text/html; charset=utf-8",
                0,
            ),
            Err(FalhaDaLupa::FormularioInesperado)
        ));
    }
}
