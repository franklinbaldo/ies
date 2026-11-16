//! Apoio dos testes de `ies unidade trocar` (#205): telas SINTÉTICAS na
//! estrutura observada nas cassettes da `UnB` 4.0.9, UFRJ 4.1.5 e GO 5.0.3
//! (barra com o par `lnkInfraUnidade`, Controle com as duas tabelas, tela
//! Trocar Unidade com `frmInfraSelecaoUnidade`), o 302 com `msg` e o 302
//! limpo, e um transporte roteirizado. Hosts, hashes, ids e siglas
//! inventados.
use super::EnvioDaTroca;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::transporte::{FalhaTransporte, RespostaWeb};
use std::collections::VecDeque;
use std::io::Cursor;
use std::path::PathBuf;

pub(crate) const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
pub(crate) const HOST: &str = "sei.exemplo.gov.br";
pub(crate) const ID_A: &str = "110000001";
pub(crate) const ID_B: &str = "110000002";
pub(crate) const ID_C: &str = "110000003";
pub(crate) const COOKIE: &str = "PHPSESSID=SINTETICO";
const ISO: &str = "text/html; charset=iso-8859-1";

pub(crate) fn hash(c: char, n: usize) -> String {
    c.to_string().repeat(n)
}

/// URL da sessão: o Controle da unidade `id` (forma do login).
pub(crate) fn url_controle(id: &str) -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&acao_origem=principal&infra_sistema=100000100&infra_unidade_atual={id}&infra_hash={}",
        hash('a', 64)
    )
}

pub(crate) fn link_relativo_da_troca(id: &str) -> String {
    format!(
        "controlador.php?acao=infra_trocar_unidade&infra_sistema=100000100&infra_unidade_atual={id}&infra_hash={}",
        hash('b', 64)
    )
}

pub(crate) fn url_da_troca(id: &str) -> String {
    format!("{ORIGEM}/{}", link_relativo_da_troca(id))
}

/// O par `lnkInfraUnidade` das barras móvel e padrão, como servido.
pub(crate) fn barra(sigla: &str, id: &str) -> String {
    let link = |tab: u32| {
        format!(
            "<a id=\"lnkInfraUnidade\" href=\"#\" onclick=\"window.location.href='{}';\" class=\"form-control\" title=\"Unidade {sigla}\" tabindex=\"{tab}\">{sigla}</a>",
            link_relativo_da_troca(id)
        )
    };
    format!(
        "<div id=\"divInfraBarraSistemaMovel\">{}</div><div id=\"divInfraBarraSistemaPadrao\">{}</div>",
        link(54),
        link(66)
    )
}

/// Controle de Processos (visão resumida) da unidade `sigla`.
pub(crate) fn controle(sigla: &str, id: &str) -> String {
    format!(
        "<html><body>{}\
         <table id=\"tblProcessosRecebidos\"><caption>0 registros:</caption></table>\
         <table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table></body></html>",
        barra(sigla, id)
    )
}

pub(crate) const SCRIPT: &str = "<script type=\"text/javascript\">\n\
    function selecionarUnidade(idUnidade){\n\
    var frm = document.getElementById('frmInfraSelecaoUnidade');\n\
    var input = document.createElement('input');\n\
    input.type = 'hidden';\n\
    input.name = 'selInfraUnidades';\n\
    input.value = idUnidade;\n\
    frm.appendChild(input);\n\
    frm.submit();\n\
    }\n</script>";

/// Uma linha da lista: rádio, Sigla, Descrição e Órgão.
pub(crate) fn linha(sigla: &str, id: &str, marcada: bool) -> String {
    let checked = if marcada { "checked=\"checked\" " } else { "" };
    format!(
        "<tr class=\"infraTrClara\"><td align=\"center\"><a id=\"lnkInfraID-{id}\" name=\"ID-{id}\"></a>\
         <input {checked}class=\"infraRadio\" id=\"chkInfraItem0\" name=\"chkInfraItem\" tabindex=\"1001\" title=\"{sigla}\" type=\"radio\" value=\"{id}\" onclick=\"infraSelecionarItens(this,'Infra'); selecionarUnidade({id})\"/></td>\
         <td align=\"center\">{sigla}</td><td>Unidade {sigla} &amp; Cia</td><td align=\"center\"><a class=\"ancoraOrgao\">ORG</a></td></tr>"
    )
}

/// A tela Trocar Unidade com a legenda declarando `total` e as `linhas`.
pub(crate) fn tela_com(ativa: (&str, &str), total: usize, linhas: &str, script: &str) -> String {
    let (sigla, id) = ativa;
    format!(
        "<html><head>{script}</head><body>{barra}\
         <form id=\"frmInfraSelecaoUnidade\" method=\"post\" onsubmit=\"return validarForm();\" \
         action=\"controlador.php?acao=infra_trocar_unidade&acao_origem=infra_trocar_unidade&infra_sistema=100000100&infra_unidade_atual={id}&infra_hash={h}\">\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"1\"/>\
         <button tabindex=\"451\" type=\"submit\" accesskey=\"P\" id=\"sbmPesquisar\" name=\"sbmPesquisar\" value=\"Pesquisar\" class=\"infraButton\">Pesquisar</button>\
         <input type=\"text\" id=\"txtInfraSiglaUnidade\" name=\"txtInfraSiglaUnidade\" class=\"infraText\" value=\"\"/>\
         <input type=\"text\" id=\"txtInfraDescricaoUnidade\" name=\"txtInfraDescricaoUnidade\" class=\"infraText\" value=\"\"/>\
         <select id=\"selInfraOrgaoUnidade\" name=\"selInfraOrgaoUnidade\" class=\"infraSelect\" onchange=\"this.form.submit()\">\
         <option value=\"null\" selected=\"selected\">&nbsp;</option><option value=\"\" selected=\"selected\">Todos</option><option value=\"0\">ORG</option></select>\
         <table width=\"99%\" class=\"infraTable\" summary=\"Tabela de Unidades com Permissão.\">\
         <caption class=\"infraCaption\">Lista de Unidades com Permissão ({total} registros):</caption>\
         <tr><th class=\"infraTh\" width=\"1%\">&nbsp;</th><th class=\"infraTh\">Sigla</th><th class=\"infraTh\">Descrição</th><th class=\"infraTh\">Órgão</th></tr>\
         {linhas}</table>\
         <input type=\"hidden\" id=\"hdnInfraNroItens\" name=\"hdnInfraNroItens\" value=\"{total}\"/>\
         <input type=\"hidden\" id=\"hdnInfraItemId\" name=\"hdnInfraItemId\" value=\"\"/>\
         <input type=\"hidden\" id=\"hdnInfraItens\" name=\"hdnInfraItens\" value=\"{ID_A},{ID_B}\"/>\
         <input type=\"hidden\" id=\"hdnInfraItensHash\" name=\"hdnInfraItensHash\" value=\"{hi}\"/>\
         <input type=\"hidden\" id=\"hdnInfraItensSelecionados\" name=\"hdnInfraItensSelecionados\" value=\"{id}\"/>\
         <input type=\"hidden\" id=\"hdnInfraSelecoes\" name=\"hdnInfraSelecoes\" value=\"Infra\"/>\
         <input type=\"hidden\" id=\"hdnInfraPaginaSelecao\" name=\"hdnInfraPaginaSelecao\" value=\"Sim\"/>\
         </form></body></html>",
        barra = barra(sigla, id),
        h = hash('c', 64),
        hi = hash('e', 64),
    )
}

/// Tela com PR-A (ativa, marcada) e PR-B.
pub(crate) fn tela() -> String {
    let linhas = format!(
        "{}{}",
        linha("PR-A", ID_A, true),
        linha("PR-B", ID_B, false)
    );
    tela_com(("PR-A", ID_A), 2, &linhas, SCRIPT)
}

/// `Location` do POST: o Controle da unidade nova com o aviso, com
/// espaços literais e hash de 192 (`esquema` `https` ou `http`, como no GO).
pub(crate) fn localizacao_com_aviso(esquema: &str, id: &str) -> String {
    format!(
        "{esquema}://{HOST}/sei/controlador.php?acao=procedimento_controlar&msg=Detectada troca de unidade PR-A para PR-B.&infra_sistema=100000100&infra_unidade_atual={id}&infra_hash={}",
        hash('f', 192)
    )
}

/// A URL que o IES pede pelo `Location` com aviso.
pub(crate) fn url_com_aviso(id: &str) -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&msg=Detectada%20troca%20de%20unidade%20PR-A%20para%20PR-B.&infra_sistema=100000100&infra_unidade_atual={id}&infra_hash={}",
        hash('f', 192)
    )
}

pub(crate) fn localizacao_limpa(esquema: &str, id: &str) -> String {
    format!(
        "{esquema}://{HOST}/sei/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual={id}&infra_hash={}",
        hash('9', 192)
    )
}

pub(crate) fn url_limpa(id: &str) -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_sistema=100000100&infra_unidade_atual={id}&infra_hash={}",
        hash('9', 192)
    )
}

/// Um pedido registrado: método, URL e, no POST, o corpo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pedido {
    pub(crate) metodo: &'static str,
    pub(crate) url: String,
    pub(crate) corpo: Option<String>,
}

pub(crate) fn get(url: impl Into<String>) -> Pedido {
    Pedido {
        metodo: "GET",
        url: url.into(),
        corpo: None,
    }
}

pub(crate) type Resposta = Result<(i32, String, Option<String>), FalhaTransporte>;

#[expect(
    clippy::unnecessary_wraps,
    reason = "O roteiro mistura respostas e falhas do transporte: `Resposta` é `Result`"
)]
pub(crate) fn html(corpo: String) -> Resposta {
    Ok((200, corpo, None))
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "O roteiro mistura respostas e falhas do transporte: `Resposta` é `Result`"
)]
pub(crate) fn redireciona(localizacao: String) -> Resposta {
    Ok((302, String::new(), Some(localizacao)))
}

/// Transporte roteirizado: uma resposta por pedido, na ordem; registra
/// cada pedido. Acabado o roteiro, um pedido a mais é pânico.
pub(crate) struct Roteiro {
    respostas: VecDeque<Resposta>,
    pub(crate) pedidos: Vec<Pedido>,
}

impl Roteiro {
    pub(crate) fn novo(respostas: Vec<Resposta>) -> Self {
        Self {
            respostas: respostas.into(),
            pedidos: Vec::new(),
        }
    }

    fn proxima(
        &mut self,
        pedido: Pedido,
    ) -> Result<RespostaDoPost<Cursor<Vec<u8>>>, FalhaTransporte> {
        let (status, corpo, localizacao) = crate::transporte::testes_http::atender(
            &mut self.pedidos,
            &mut self.respostas,
            pedido,
        )?;
        Ok(RespostaDoPost {
            resposta: RespostaWeb {
                status_code: status,
                mime: ISO.to_owned(),
                content_length: None,
                localizacao: None,
                // Servido em ISO-8859-1, como nas instalações.
                corpo: Cursor::new(
                    corpo
                        .chars()
                        .map(|c| u8::try_from(u32::from(c)).expect("latin-1"))
                        .collect(),
                ),
            },
            localizacao,
        })
    }
}

impl EnvioDaTroca for Roteiro {
    type Corpo = Cursor<Vec<u8>>;
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, COOKIE);
        self.proxima(get(url))
    }
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        assert_eq!(cookie, COOKIE);
        self.proxima(Pedido {
            metodo: "POST",
            url: submissao.url().to_owned(),
            corpo: Some(String::from_utf8(submissao.corpo().to_vec()).unwrap()),
        })
    }
}

/// Arquivo de cookie privado (0600) num diretório temporário próprio.
pub(crate) fn arquivo_de_cookie(nome: &str) -> PathBuf {
    let pasta = std::env::temp_dir().join(format!(
        "ies-trocar-{nome}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    let caminho = pasta.join("cookie");
    std::fs::write(&caminho, COOKIE).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    caminho
}
