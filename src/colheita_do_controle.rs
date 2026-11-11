//! Colheita do Controle no índice local
//! ([ADR 0003](../docs/arquitetura/0003-indice-local.md)): cada página do
//! Controle de Processos que um comando **já** leu alimenta o índice com o
//! que ela mostra, sem nenhuma requisição. Por linha válida, NUP ↔ id e o
//! `href` assinado de `procedimento_trabalhar`; da página, o link do menu
//! `protocolo_pesquisar` e o `action` de `frmProtocoloPesquisaRapida`.
//!
//! As linhas vêm dos leitores da caixa (R: `caixa::linhas_com_link`; D:
//! `caixa_detalhada::linhas_com_link`); não há segundo leitor de linhas.
//! Nada aqui falha o comando: linha, página ou banco ruins só não gravam.
use crate::acao_do_sei::AcaoDoSei;
use crate::caixa::LinhaDoControle;
use crate::controle_contexto::contexto_da_pagina;
use crate::controle_reconhecido::publica_visao_detalhada;
use crate::formulario_web::{TelaServida, decodificar_atributo, derivar_submissao};
use crate::indice_local::{ChaveDoLink, IndiceLocal, unidade_dos_links};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::pesquisa_web::link_da_pesquisa::link_da_pesquisa;
use crate::pesquisa_web::redirecionamento::{OrigemDoTrabalho, conferir_trabalho};
use crate::processo_obter::{CAMPO_DA_PESQUISA_RAPIDA, PESQUISA_RAPIDA};
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
use std::collections::HashMap;

#[cfg(test)]
#[path = "colheita_do_controle/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "colheita_do_controle/tests_avisos.rs"]
mod tests_avisos;

/// Para onde e com que chave colher: índice, instalação, geração da sessão
/// (SHA-256 do cookie) e a URL privada do Controle, de onde vêm a unidade
/// da chave e a base dos `href` relativos. Sem `Debug`: geração e URL
/// derivam da sessão.
pub(crate) struct Colheita<'i> {
    alvo: Option<Alvo<'i>>,
}

struct Alvo<'i> {
    indice: &'i IndiceLocal,
    instalacao: Instalacao,
    geracao: String,
    url_do_controle: String,
}

impl<'i> Colheita<'i> {
    /// Nada é colhido.
    pub(crate) fn nenhuma() -> Self {
        Self { alvo: None }
    }

    /// A geração já calculada (a mesma de `ies processo NUP`).
    pub(crate) fn com_geracao(
        indice: &'i IndiceLocal,
        instalacao: &Instalacao,
        geracao: &str,
        url_do_controle: &str,
    ) -> Self {
        if !indice.ligado() {
            return Self::nenhuma();
        }
        Self {
            alvo: Some(Alvo {
                indice,
                instalacao: instalacao.clone(),
                geracao: geracao.to_owned(),
                url_do_controle: url_do_controle.to_owned(),
            }),
        }
    }

    /// A geração do cookie em uso (`geracao_da_sessao`).
    pub(crate) fn do_cookie(
        indice: &'i IndiceLocal,
        instalacao: &Instalacao,
        cookie: &str,
        url_do_controle: &str,
    ) -> Self {
        let geracao = crate::sessao_login::geracao_da_sessao(cookie);
        Self::com_geracao(indice, instalacao, &geracao, url_do_controle)
    }

    /// A colheita do retrato coerente da sessão (#222) que o comando usa
    /// nas requisições: a chave é a geração do cookie que obtém a página,
    /// nunca a de arquivos relidos depois.
    pub(crate) fn das_referencias(
        indice: &'i IndiceLocal,
        sessao: &crate::transporte::sessao::ReferenciasDaSessao,
    ) -> Self {
        if !indice.ligado() {
            return Self::nenhuma();
        }
        let Some(instalacao) = Instalacao::interpretar(sessao.origem()) else {
            return Self::nenhuma();
        };
        let Ok(cookie) = sessao.cookie().valor() else {
            return Self::nenhuma();
        };
        Self::do_cookie(indice, &instalacao, cookie, sessao.url_do_controle())
    }

    /// Colhe uma página do Controle já lida. `mime` é o `Content-Type` da
    /// resposta (o charset do `action` da pesquisa rápida); sem ele, a
    /// pesquisa rápida não é colhida.
    pub(crate) fn colher(&self, html: &str, mime: Option<&str>) {
        let Some(alvo) = &self.alvo else {
            return;
        };
        let Some(unidade) = unidade_dos_links(&alvo.url_do_controle) else {
            return;
        };
        let (indice, instalacao) = (alvo.indice, &alvo.instalacao);
        // Só publica o fato após o leitor REAL validar o Controle R/D.
        // O índice armazena a geração derivada do cookie, nunca o cookie.
        if let Ok(nome) = crate::controle_reconhecido::unidade_do_controle(html)
            && let Some(observada) = iesei::observacao::UnidadeExibida::nova(&nome)
        {
            let visao = if publica_visao_detalhada(html) {
                iesei::observacao::VisaoDoControle::Detalhada
            } else {
                iesei::observacao::VisaoDoControle::Resumida
            };
            indice.registrar_unidade_observada(instalacao, &alvo.geracao, &observada, visao);
        }
        for (linha, url) in linhas_conferidas(alvo, html) {
            let gravado = registrar(
                alvo,
                &unidade,
                AcaoDoSei::ProcedimentoTrabalhar,
                &linha.id,
                &url,
            );
            if gravado {
                indice.registrar_processo(instalacao, &linha.numero, &linha.id);
            }
        }
        if let Ok(url) = link_da_pesquisa(instalacao, &alvo.url_do_controle, html) {
            registrar(alvo, &unidade, AcaoDoSei::ProtocoloPesquisar, "", &url);
        }
        if let Some(url) = mime.and_then(|mime| action_da_pesquisa_rapida(alvo, html, mime)) {
            registrar(alvo, &unidade, PESQUISA_RAPIDA.acao, "", &url);
        }
    }
}

/// Guarda o link sem editar; a recusa da conferência fica nos avisos do
/// índice. `true` se a entrada passou.
fn registrar(alvo: &Alvo<'_>, unidade: &str, acao: AcaoDoSei, id_alvo: &str, url: &str) -> bool {
    let chave = ChaveDoLink {
        geracao_sessao: &alvo.geracao,
        unidade,
        acao,
        alvo: id_alvo,
    };
    alvo.indice.guardar_link(&alvo.instalacao, chave, url)
}

/// As linhas da página pelo leitor da visão que ela publica (D se houver
/// `tblProcessosDetalhado`, senão R); página recusada: nenhuma.
fn linhas_da_pagina(html: &str) -> Vec<LinhaDoControle> {
    if publica_visao_detalhada(html) {
        crate::caixa_detalhada::linhas_com_link(html).unwrap_or_default()
    } else {
        crate::caixa::linhas_com_link(html).unwrap_or_default()
    }
}

/// Cada linha de NUP e id únicos na página cujo `href` relativo é o
/// `procedimento_trabalhar` do Controle para o id da linha, com a URL
/// absoluta conferida. As outras são puladas.
fn linhas_conferidas(alvo: &Alvo<'_>, html: &str) -> Vec<(LinhaDoControle, String)> {
    let linhas = linhas_da_pagina(html);
    let mut nups: HashMap<&str, usize> = HashMap::new();
    let mut ids: HashMap<&str, usize> = HashMap::new();
    for linha in &linhas {
        *nups.entry(&linha.numero).or_default() += 1;
        *ids.entry(&linha.id).or_default() += 1;
    }
    let unicas: Vec<bool> = linhas
        .iter()
        .map(|l| nups.get(l.numero.as_str()) == Some(&1) && ids.get(l.id.as_str()) == Some(&1))
        .collect();
    linhas
        .into_iter()
        .zip(unicas)
        .filter(|(_, unica)| *unica)
        .filter_map(|(linha, _)| {
            let url = trabalho_da_linha(alvo, &linha)?;
            Some((linha, url))
        })
        .collect()
}

fn trabalho_da_linha(alvo: &Alvo<'_>, linha: &LinhaDoControle) -> Option<String> {
    let href = decodificar_atributo(linha.href.as_deref()?)?;
    if !href.starts_with("controlador.php?") {
        return None;
    }
    let origem = OrigemDoTrabalho::Controle { id: &linha.id };
    conferir_trabalho(Some(&href), &alvo.url_do_controle, &alvo.instalacao, origem)
        .ok()
        .map(|conferido| conferido.trabalho.as_str().to_owned())
}

/// O `action` assinado de `frmProtocoloPesquisaRapida`, pela mesma
/// derivação do envio de `ies processo NUP`.
fn action_da_pesquisa_rapida(alvo: &Alvo<'_>, html: &str, mime: &str) -> Option<String> {
    let codificacao = codificacao_da_tela(mime)?;
    let unidade = contexto_da_pagina(html).ok()?.unidade?;
    let tela = TelaServida {
        url: &alvo.url_do_controle,
        html,
        codificacao,
    };
    derivar_submissao(
        &PESQUISA_RAPIDA,
        &alvo.instalacao,
        &tela,
        &unidade,
        &[(CAMPO_DA_PESQUISA_RAPIDA, "")],
    )
    .ok()
    .map(|submissao| submissao.url().to_owned())
}

/// O envio HTTPS real que colhe cada tela do Controle que os fluxos de
/// visão e página recebem (`EnvioDoControle::colher_controle`).
pub(crate) struct EnvioColhendo<'c, 'i> {
    pub(crate) colheita: &'c Colheita<'i>,
}

impl EnvioDoControle for EnvioColhendo<'_, '_> {
    type Corpo = <EnvioHttps as EnvioDoControle>::Corpo;
    fn post(
        &mut self,
        submissao: crate::formulario_web::SubmissaoDerivada,
        cookie: &str,
    ) -> Result<
        crate::formulario_web::RespostaDoPost<Self::Corpo>,
        crate::transporte::FalhaTransporte,
    > {
        EnvioHttps.post(submissao, cookie)
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<crate::transporte::RespostaWeb<Self::Corpo>, crate::transporte::FalhaTransporte>
    {
        EnvioHttps.get(url, cookie)
    }
    fn colher_controle(&mut self, html: &str, mime: Option<&str>) {
        self.colheita.colher(html, mime);
    }
}
