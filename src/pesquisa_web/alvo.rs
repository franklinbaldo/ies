//! As duas pesquisas da mesma tela `frmPesquisaProtocolo`: em processos
//! (`ies processo buscar`) e em documentos (`ies documento buscar`). O
//! fluxo, o formulário e o transporte são os mesmos; mudam o rádio (e as
//! caixas de tipo), o leitor das linhas e a forma do 302.
use super::formulario_pesquisa::{EnvioInterpretado, ErroPesquisaProtocolo, interpretar_envio};
use super::paginacao::Contagem;
use super::regras_formulario::PesquisarEm;
use super::resposta::{
    ErroRespostaDaPesquisa, FaixaDaPesquisa, RespostaDaPesquisa, ler_resposta_na_pagina,
};
use crate::formulario_web::RespostaDoEnvio;
use crate::instalacao::Instalacao;

/// O que a operação lê da resposta do envio e de uma página.
pub(crate) trait AlvoDaPesquisa {
    type Resposta;
    const EM: PesquisarEm;
    /// O 302 conferido (nunca seguido) ou o 200 classificado.
    fn interpretar(
        enviada: RespostaDoEnvio,
        instalacao: &Instalacao,
    ) -> Result<EnvioInterpretado<Self::Resposta>, ErroPesquisaProtocolo>;
    /// A resposta 200 do POST de página, com a faixa esperada e o número.
    fn ler_pagina(
        html: &crate::tela_html::HtmlServido,
        na_pagina: (FaixaDaPesquisa, u64),
    ) -> Result<Self::Resposta, ErroRespostaDaPesquisa>;
    fn contagem(resposta: &Self::Resposta) -> Contagem;
}

/// `ies processo buscar`: `#optProcessos`, linhas de processo.
pub(crate) struct EmProcessos;

impl AlvoDaPesquisa for EmProcessos {
    type Resposta = RespostaDaPesquisa;
    const EM: PesquisarEm = PesquisarEm::Processos;
    fn interpretar(
        enviada: RespostaDoEnvio,
        instalacao: &Instalacao,
    ) -> Result<EnvioInterpretado, ErroPesquisaProtocolo> {
        interpretar_envio(enviada, instalacao)
    }
    fn ler_pagina(
        html: &crate::tela_html::HtmlServido,
        na_pagina: (FaixaDaPesquisa, u64),
    ) -> Result<RespostaDaPesquisa, ErroRespostaDaPesquisa> {
        ler_resposta_na_pagina(html, Some(na_pagina))
    }
    fn contagem(resposta: &RespostaDaPesquisa) -> Contagem {
        resposta.contagem()
    }
}
