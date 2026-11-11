//! Telas sintéticas R, D e do Nível de detalhe, modeladas na estrutura das
//! capturas privadas de `UnB`, PEN1 e GO (02/11/2026), com unidade, ids,
//! números, nomes e hashes fictícios, e um envio injetável que registra a
//! sequência de requisições.
use super::envio::EnvioDoControle;
use super::*;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::instalacao::Instalacao;
use crate::tela_html::HtmlServido;
use crate::transporte::RespostaWeb;
use std::fmt::Write as _;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

pub(super) const URL_GET: &str = "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar&infra_hash=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_FORM: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_NIVEL: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

pub(super) fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
}

fn formulario(ocultos: &[(&str, &str)], extra: &str, tabelas: &str) -> String {
    let mut campos = String::new();
    for (nome, valor) in ocultos {
        write!(
            campos,
            "<input type='hidden' id='{nome}' name='{nome}' value='{valor}'>"
        )
        .unwrap();
    }
    format!(
        "<form id='frmProcedimentoControlar' method='post' \
         action='controlador.php?acao=procedimento_controlar&amp;acao_origem=procedimento_controlar&amp;infra_hash={HASH_FORM}'>\
         {campos}<input type='text' id='txtBloco' name='txtBloco' value=''>{extra}{tabelas}</form>"
    )
}

fn linhas_r(ids: &[u32]) -> String {
    ids.iter().fold(String::new(), |mut s, id| {
        write!(
            s,
            "<tr id='P{id}'><td><a class='processoVisualizado' \
             href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento={id}'>N-{id}</a></td></tr>"
        )
        .unwrap();
        s
    })
}

/// Tela R: Recebidos e Gerados com a página e os ids dados.
#[derive(Clone)]
pub(super) struct TelaR {
    pub(super) unidade: &'static str,
    pub(super) recebidos: (usize, usize, Vec<u32>),
    pub(super) gerados: (usize, usize, Vec<u32>),
    pub(super) pagina_r: &'static str,
    pub(super) pagina_g: &'static str,
    pub(super) hash_r: &'static str,
}

impl TelaR {
    pub(super) fn nova(recebidos: (usize, Vec<u32>), gerados: (usize, Vec<u32>)) -> Self {
        Self {
            unidade: "TESTE",
            recebidos: (recebidos.0, 1, recebidos.1),
            gerados: (gerados.0, 1, gerados.1),
            pagina_r: "0",
            pagina_g: "0",
            hash_r: "cccc",
        }
    }

    fn tabela(nome: &str, (total, inicio, ids): &(usize, usize, Vec<u32>)) -> String {
        let legenda = if ids.is_empty() {
            format!("{total} registros:")
        } else {
            format!(
                "({total} registros - {inicio} a {}):",
                inicio + ids.len() - 1
            )
        };
        let paginador = if ids.len() < *total {
            format!("<a id='lnk{nome}ProximaPaginaSuperior'>»</a>")
        } else {
            String::new()
        };
        format!(
            "{paginador}<table id='tblProcessos{nome}' class='infraTable tabelaControle'>\
             <caption>{legenda}</caption>{}</table>",
            linhas_r(ids)
        )
    }

    pub(super) fn html(&self) -> String {
        let tabelas = format!(
            "{}{}",
            Self::tabela("Recebidos", &self.recebidos),
            Self::tabela("Gerados", &self.gerados)
        );
        let form = formulario(
            &[
                ("hdnRecebidosPaginaAtual", self.pagina_r),
                ("hdnGeradosPaginaAtual", self.pagina_g),
                ("hdnRecebidosHashCriterios", self.hash_r),
                ("hdnGeradosHashCriterios", "dddd"),
                ("hdnTipoVisualizacao", "R"),
            ],
            "",
            &tabelas,
        );
        format!(
            "<html><body><a id='lnkInfraUnidade'>{}</a>{form}</body></html>",
            self.unidade
        )
    }
}

/// Cabeçalho D no estilo de GO/PEN1: seleção, ícones, Processo, Atribuição,
/// Tipo, Interessados, Controle de Prazo (ícone), Última Mov., Marcadores
/// (forma da `UnB`; dois no processo 1, nenhum nos outros) e ações.
pub(super) const CABECALHO_D: &str = "<tr>\
    <th class='infraTh'><a id='lnkInfraCheck'><img title='Selecionar Tudo' class='infraImg'></a></th>\
    <th class='infraTh'>&nbsp;</th>\
    <th class='infraTh'><div class='infraDivOrdenacao'><div class='infraDivRotuloOrdenacao'>Processo</div>\
      <div class='infraDivSetaOrdenacao'><a><img title='Ordenar Processo' class='infraImgOrdenacao'></a></div></div></th>\
    <th class='infraTh'><div class='infraDivOrdenacao'><div class='infraDivRotuloOrdenacao'>Atribuição</div></div></th>\
    <th class='infraTh'><div class='infraDivOrdenacao'><div class='infraDivRotuloOrdenacao'>Tipo</div></div></th>\
    <th class='infraTh'>Interessados</th>\
    <th class='infraTh'><div class='infraDivOrdenacao'><div class='infraDivRotuloOrdenacao'>\
      <img aria-label='Controle de Prazo' onmouseover=\"return infraTooltipMostrar('Controle de Prazo');\" class='infraImg'></div></div></th>\
    <th class='infraTh'><div class='infraDivOrdenacao'><div class='infraDivRotuloOrdenacao'>Última&nbsp;Mov.</div></div></th>\
    <th class='infraTh'>Marcadores</th>\
    <th class='infraTh'>&nbsp;</th></tr>";

pub(super) fn linha_d(id: u32) -> String {
    use crate::caixa_detalhada::tests_marcadores::ancora;
    let marcadores = if id == 1 {
        format!(
            "{}{}",
            ancora("1", "31", "amarelo", "Texto sintético", "Marcador A"),
            ancora("1", "32", "azul", "", "Marcador B")
        )
    } else {
        "&nbsp;".to_owned()
    };
    format!(
        "<tr id='P{id}'><td><input type='checkbox' id='chkDetalhadoItem{id}' name='chkDetalhadoItem{id}' value='{id}'></td>\
         <td data-label='&nbsp;'></td>\
         <td data-label='Processo'><a class='processoVisualizado' \
           href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento={id}'>N-{id}</a></td>\
         <td data-label='Atribuição'><a class='ancoraSigla'>pessoa{id}</a></td>\
         <td data-label='Tipo'>  Tipo   sintético </td>\
         <td data-label='Interessados'><div>\
           <div class='divItemCelula'><div class='divIconeItemCelula'><img></div><div class='divRotuloItemCelula'>Interessado A</div></div>\
           <div class='divItemCelula'><div class='divIconeItemCelula'><img></div><div class='divRotuloItemCelula'>Interessado&nbsp;B</div></div>\
         </div></td>\
         <td></td><td data-label='Última Mov.'>3</td>\
         <td align=\"left\" valign=\"top\">{marcadores}</td><td data-label='&nbsp;'></td></tr>"
    )
}

/// Tela D: total, índice da página (`None`: sem `hdnDetalhadoPaginaAtual`,
/// como a `UnB`) e ids exibidos.
#[derive(Clone)]
pub(super) struct TelaD {
    pub(super) unidade: &'static str,
    pub(super) total: usize,
    pub(super) indice: Option<usize>,
    pub(super) tamanho: usize,
    pub(super) ids: Vec<u32>,
    pub(super) cabecalho: String,
    pub(super) criterio: &'static str,
    pub(super) link_nivel: bool,
}

impl TelaD {
    pub(super) fn nova(total: usize, indice: usize, tamanho: usize, ids: Vec<u32>) -> Self {
        Self {
            unidade: "TESTE",
            total,
            indice: Some(indice),
            tamanho,
            ids,
            cabecalho: CABECALHO_D.to_owned(),
            criterio: "ffff",
            link_nivel: true,
        }
    }

    pub(super) fn html(&self) -> String {
        let inicio = self.indice.unwrap_or(0) * self.tamanho + 1;
        let legenda = if self.ids.is_empty() {
            format!("Processos ({} registros):", self.total)
        } else {
            format!(
                "Processos ({} registros - {inicio} a {}):",
                self.total,
                inicio + self.ids.len() - 1
            )
        };
        let linhas: String = self.ids.iter().map(|id| linha_d(*id)).collect();
        let tabela = format!(
            "<table id='tblProcessosDetalhado' class='infraTableResponsiva infraTable tabelaControle'>\
             <caption class='infraCaption'>{legenda}</caption>{}{linhas}</table>",
            self.cabecalho
        );
        let mut ocultos = vec![
            ("hdnDetalhadoHashCriterios", self.criterio),
            ("hdnTipoVisualizacao", "D"),
        ];
        let indice = self.indice.map(|i| i.to_string());
        let mut extra = String::new();
        if let Some(indice) = indice.as_deref() {
            ocultos.insert(0, ("hdnDetalhadoPaginaAtual", indice));
            if self.total > self.tamanho {
                extra = format!(
                    "<a id='lnkDetalhadoProximaPaginaSuperior'>»</a>\
                     <select id='selDetalhadoPaginacaoSuperior' name='selDetalhadoPaginacaoSuperior'>\
                     <option value='0'>1</option><option value='{indice}' selected>{}</option></select>",
                    self.indice.unwrap_or(0) + 1
                );
            }
        }
        let form = formulario(&ocultos, &extra, &tabela);
        let link = if self.link_nivel {
            format!(
                "<a id=\"ancNivelDetalhe\" href=\"javascript:void(0);\" \
                 onclick=\"acaoConfigurarDetalhe('controlador.php?acao=procedimento_configurar_detalhe&amp;acao_origem=procedimento_controlar&amp;acao_retorno=procedimento_controlar&amp;infra_sistema=100&amp;infra_unidade_atual=200&amp;infra_hash={HASH_NIVEL}');\" \
                 class=\"ancoraPadraoPreta p-0\">Configurar detalhe</a>"
            )
        } else {
            String::new()
        };
        format!(
            "<html><body><a id='lnkInfraUnidade'>{}</a>{link}{form}</body></html>",
            self.unidade
        )
    }
}

/// Tela do Nível de detalhe com as opções marcadas.
pub(super) fn tela_nivel(marcadas: &[&str]) -> String {
    crate::nivel_de_detalhe::tests::tela(marcadas)
}

pub(super) struct Pasta(PathBuf);

impl Drop for Pasta {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(super) fn cookie() -> (Pasta, PathBuf) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let pasta = std::env::temp_dir().join(format!(
        "ies-visao-controle-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&pasta).unwrap();
    let arquivo = pasta.join("cookie");
    fs::write(&arquivo, "PHPSESSID=SINTETICO\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
    }
    (Pasta(pasta), arquivo)
}

/// Envio roteirizado: cada requisição consome a próxima resposta e é
/// registrada como `POST visao=X [pagina=I] [recebidos=I]` ou `GET acao`.
pub(super) struct EnvioRoteirizado {
    pub(super) respostas: Vec<(i32, String)>,
    pub(super) registro: Vec<String>,
}

impl EnvioRoteirizado {
    pub(super) fn novo(respostas: Vec<(i32, String)>) -> Self {
        Self {
            respostas,
            registro: Vec::new(),
        }
    }

    fn proxima(&mut self) -> RespostaWeb<Cursor<Vec<u8>>> {
        assert!(!self.respostas.is_empty(), "requisição além do orçamento");
        let (status_code, html) = self.respostas.remove(0);
        // O corpo vai em ISO-8859-1, como o `Content-Type` declara.
        let corpo = html
            .chars()
            .map(|c| u8::try_from(u32::from(c)).expect("ISO-8859-1"))
            .collect();
        crate::transporte::testes_http::resposta(
            status_code,
            "text/html; charset=iso-8859-1",
            corpo,
            None,
        )
    }
}

fn valor_no_corpo<'a>(corpo: &'a str, nome: &str) -> Option<&'a str> {
    corpo
        .split('&')
        .find_map(|par| par.strip_prefix(nome)?.strip_prefix('='))
}

impl EnvioDoControle for EnvioRoteirizado {
    type Corpo = Cursor<Vec<u8>>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, crate::transporte::FalhaTransporte> {
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        assert!(submissao.url().contains(HASH_FORM));
        let corpo = String::from_utf8(submissao.corpo().to_vec()).unwrap();
        let mut linha = format!(
            "POST visao={}",
            valor_no_corpo(&corpo, "hdnTipoVisualizacao").unwrap_or("?")
        );
        for (campo, rotulo) in [
            ("hdnDetalhadoPaginaAtual", "pagina"),
            ("hdnRecebidosPaginaAtual", "recebidos"),
            ("hdnGeradosPaginaAtual", "gerados"),
        ] {
            if let Some(valor) = valor_no_corpo(&corpo, campo) {
                write!(linha, " {rotulo}={valor}").unwrap();
            }
        }
        self.registro.push(linha);
        Ok(RespostaDoPost {
            resposta: self.proxima(),
            localizacao: None,
        })
    }

    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, crate::transporte::FalhaTransporte> {
        assert_eq!(cookie, "PHPSESSID=SINTETICO");
        assert!(url.starts_with("https://sei.exemplo.gov.br/sei/controlador.php?"));
        let acao = url
            .split(['?', '&'])
            .find_map(|p| p.strip_prefix("acao="))
            .unwrap_or("?")
            .to_owned();
        self.registro.push(format!("GET {acao}"));
        Ok(self.proxima())
    }
}

pub(super) fn tela_atual(html: String) -> TelaAtual {
    TelaAtual {
        url: URL_GET.to_owned(),
        html: HtmlServido::novo(html).unwrap(),
        mime: "text/html; charset=iso-8859-1".to_owned(),
    }
}
