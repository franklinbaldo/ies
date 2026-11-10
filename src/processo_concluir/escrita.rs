//! Peças das escritas de estado do processo pela barra da árvore, comuns a
//! `processo concluir` e `processo reabrir` (contracts/cli/processo-concluir.md,
//! contracts/cli/processo-reabrir.md): a contagem de requisições, as siglas
//! de `aberto_em`, o cookie, o GET de uma tela antes da escrita e o GET que
//! é a própria escrita (a URL do corpo de uma função da tela do nó).
use super::erro::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::arvore_processo::{AbertoEm, MotivoAbertoEm};
use crate::formulario_web::{ErroSubmissaoFormulario, RespostaDoPost, SubmissaoDerivada};
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_obter::{
    EnvioDoProcesso, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso, obter_processo,
};
use crate::transporte::validacao_da_infra::mensagem_de_validacao_da_infra;
use crate::transporte::{FalhaTransporte, RespostaWeb, decodificar_resposta};
use crate::visao_do_controle::EnvioDoControle;

/// As siglas de `aberto_em`, ou o motivo de não ter sido lido.
pub(crate) fn siglas_abertas(aberto_em: &AbertoEm) -> Result<Vec<String>, MotivoAbertoEm> {
    match aberto_em {
        AbertoEm::Unidades(unidades) => Ok(unidades.iter().map(|u| u.sigla.clone()).collect()),
        AbertoEm::Sobrestado(_) => Err(MotivoAbertoEm::RaizPublicaOSobrestamento),
        AbertoEm::NaoLido(motivo) => Err(*motivo),
    }
}

/// A leitura de `ies processo NUP` sem extras. Com `unidade_exigida`, a da
/// abertura (sem o link guardado da árvore, que não publica a unidade);
/// sem, a de verificação (o passo mais barato).
pub(crate) fn leitura<'a>(
    nup: &'a NumeroProcesso,
    unidade: Option<&'a str>,
    unidade_exigida: bool,
) -> PedidoDoProcesso<'a> {
    PedidoDoProcesso {
        nup,
        unidade,
        andamentos: PedidoDeAndamentos::Omitidos,
        propriedades: false,
        marcadores: false,
        unidade_exigida,
        leitura_das_pastas: crate::processo_obter::LeituraDasPastas::Estrita,
    }
}

/// A leitura nova da árvore depois da escrita, feita uma vez.
pub(crate) fn verificar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoProcesso<'_>,
    indice: &IndiceLocal,
    rede: &mut Contador<'_, V>,
) -> VerificacaoPorLeitura {
    match obter_processo(sessao, pedido, indice, rede) {
        Ok(lido) => match siglas_abertas(&lido.arvore.aberto_em) {
            Ok(siglas) => VerificacaoPorLeitura::Lida(siglas),
            Err(motivo) => VerificacaoPorLeitura::SituacaoNaoLida(motivo),
        },
        Err(erro) => VerificacaoPorLeitura::LeituraFalhou(erro),
    }
}

/// O cookie do retrato da sessão (#222), lido do arquivo privado e conferido.
/// O cookie privado da sessão não pôde ser lido, antes de qualquer
/// requisição. Tipo próprio da etapa: os fluxos o recebem por `From` como
/// erro `Cookie`, distinto do transporte das telas.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum CookieIlegivel {
    Arquivo(#[from] FalhaTransporte),
}

pub(crate) fn ler_cookie(sessao: &SessaoDoProcesso<'_>) -> Result<String, CookieIlegivel> {
    Ok(sessao.cookie.valor()?.to_owned())
}

/// Por que uma tela pedida antes da escrita não foi servida.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDaTela {
    Transporte(#[from] FalhaTransporte),
    Redirecionada,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
}

/// GET de uma tela antes da escrita (a de conclusão ou a do nó): HTML 200.
pub(crate) fn obter_tela<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
) -> Result<(String, String), FalhaDaTela> {
    let resposta = rede.get_sem_seguir(url, cookie)?;
    if resposta.resposta.status_code == 302 {
        return Err(FalhaDaTela::Redirecionada);
    }
    let mime = resposta.resposta.mime.clone();
    match decodificar_resposta(resposta.resposta) {
        Ok(html) => Ok((html, mime)),
        Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle) => Err(FalhaDaTela::SessaoEncerrada(
            crate::erro_publico::SessaoEncerrada,
        )),
        Err(falha) => Err(FalhaDaTela::Transporte(falha)),
    }
}

/// O GET da URL de uma função da tela do nó, que é a escrita. `None`: HTML
/// 200 sem login nem validação da Infra; um 302 vira `redirecionada`.
pub(crate) fn pela_funcao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
    redirecionada: FalhaDoEnvio,
) -> Option<FalhaDoEnvio> {
    match rede.get_sem_seguir(url, cookie) {
        Ok(resposta) => resposta_da_funcao(resposta, redirecionada),
        Err(falha) => Some(FalhaDoEnvio::Transporte(falha)),
    }
}

/// A resposta do GET que é a escrita, sem rede (também a do replay do
/// acervo). `None`: a esperada.
pub(crate) fn resposta_da_funcao<R: std::io::Read>(
    resposta: RespostaDoPost<R>,
    redirecionada: FalhaDoEnvio,
) -> Option<FalhaDoEnvio> {
    if resposta.resposta.status_code == 302 {
        return Some(redirecionada);
    }
    match decodificar_resposta(resposta.resposta) {
        Ok(html) => mensagem_de_validacao_da_infra(&html).map(|mensagem| {
            FalhaDoEnvio::Envio(ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(mensagem))
        }),
        Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle) => {
            Some(FalhaDoEnvio::LoginAposEscrita)
        }
        Err(falha) => Some(FalhaDoEnvio::Transporte(falha)),
    }
}

/// O transporte com a contagem de todas as requisições do comando
/// (inclusive as que falharam), da abertura à verificação.
pub(crate) struct Contador<'v, V> {
    envio: &'v mut V,
    feitas: crate::transporte::contagem::ContagemDeRequisicoes,
}

impl<'v, V> Contador<'v, V> {
    pub(crate) fn novo(envio: &'v mut V) -> Self {
        Self {
            envio,
            feitas: crate::transporte::contagem::ContagemDeRequisicoes::default(),
        }
    }

    pub(crate) fn feitas(&self) -> u8 {
        self.feitas.total_u8()
    }
}

impl<V: EnvioDoProcesso> EnvioDoControle for Contador<'_, V> {
    type Corpo = V::Corpo;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.feitas.registrar();
        self.envio.post(submissao, cookie)
    }
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaWeb<Self::Corpo>, FalhaTransporte> {
        self.feitas.registrar();
        self.envio.get(url, cookie)
    }
    fn colher_controle(&mut self, html: &str, mime: Option<&str>) {
        self.envio.colher_controle(html, mime);
    }
}

impl<V: EnvioDoProcesso> EnvioDoProcesso for Contador<'_, V> {
    fn get_sem_seguir(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        self.feitas.registrar();
        self.envio.get_sem_seguir(url, cookie)
    }
    fn get_binario(
        &mut self,
        url: &str,
        cookie: &str,
        limite: usize,
    ) -> Result<crate::transporte::resposta_binaria::RespostaBinaria<Self::Corpo>, FalhaTransporte>
    {
        self.feitas.registrar();
        self.envio.get_binario(url, cookie, limite)
    }
}
