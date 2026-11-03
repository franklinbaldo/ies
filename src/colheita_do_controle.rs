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
use crate::configuracao::ArquivosDeAcesso;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};

#[cfg(test)]
#[path = "colheita_do_controle/tests.rs"]
mod tests;

/// Ação do link de cada linha (objeto = `id_procedimento`).
pub(crate) const ACAO_DO_TRABALHO: &str = "procedimento_trabalhar";
/// Ação do link do menu da pesquisa (objeto vazio).
pub(crate) const ACAO_DA_PESQUISA: &str = "protocolo_pesquisar";

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

    /// Lê origem, URL do Controle e cookie dos arquivos privados (antes do
    /// GET, para a chave ser a do cookie que o GET vai usar). Qualquer
    /// falha: nada é colhido; o comando falha, se for o caso, pela leitura dele.
    pub(crate) fn da_sessao(indice: &'i IndiceLocal, arquivos: &ArquivosDeAcesso) -> Self {
        use crate::transporte::arquivo_privado;
        if !indice.ligado() {
            return Self::nenhuma();
        }
        let lidos = arquivo_privado(&arquivos.origem, 512).and_then(|origem| {
            let url = arquivo_privado(&arquivos.url_controle, 8_192)?;
            let cookie = arquivo_privado(&arquivos.cookie, 8_192)?;
            Ok((origem, url, cookie))
        });
        let Ok((origem, url, cookie)) = lidos else {
            return Self::nenhuma();
        };
        let Some(instalacao) = Instalacao::interpretar(origem.trim()) else {
            return Self::nenhuma();
        };
        Self::do_cookie(indice, &instalacao, cookie.trim(), url.trim())
    }

    /// Colhe uma página do Controle já lida. `mime` é o `Content-Type` da
    /// resposta (o charset do `action` da pesquisa rápida); sem ele, a
    /// pesquisa rápida não é colhida.
    pub(crate) fn colher(&self, html: &str, mime: Option<&str>) {
        let Some(alvo) = &self.alvo else {
            return;
        };
        let _ = (alvo, html, mime);
    }
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
