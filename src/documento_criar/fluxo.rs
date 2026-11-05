//! Prévia Web somente de leitura da inclusão de documento interno (#246).
//! Nunca envia o formulário de criação e não faz retry.
use super::{
    ErroCriar, conferir_cadastro, conferir_criacao, conferir_tipo, link_direto_do_tipo,
    tipos_oferecidos,
};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    derivar_submissao, enviar_submissao_via,
};
use crate::arvore_processo::ErroLinkIncluirDocumento;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{
    Contador, FalhaDaTela, ler_cookie, leitura, obter_tela, siglas_abertas,
};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever, obter_processo,
};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};
use crate::visao_do_controle::EnvioDoControle as _;

// A forma de documento_gerar observada no Todos 323eb01 e em
// docs/como-o-sei-funciona/inclusao-documento-interno.md: submit() reescreve
// hdnFlagDocumentoCadastro para "2". Nenhum JavaScript é executado.
const CADASTRAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "documento_gerar",
    formulario: "frmDocumentoCadastro",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnFlagDocumentoCadastro", "hdnIdSerie"],
    campos_ocultos_reescritos: &["hdnFlagDocumentoCadastro"],
    campos_usuario: &["txtDescricao", "rdoNivelAcesso"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::DocumentoCriadoNoProcesso,
    },
};

pub(crate) struct PedidoCriacao<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) tipo: &'a str,
    pub(crate) descricao: Option<&'a str>,
    pub(crate) nivel: Option<&'a str>,
}

pub(crate) struct PreviaCriacao {
    pub(crate) nup: String,
    pub(crate) tipo: String,
    pub(crate) unidade: String,
    pub(crate) requisicoes: u8,
    pub(crate) confirmado: bool,
    pub(crate) numero_sei: Option<String>,
}

pub(crate) enum ErroPreviaCriacao {
    Abertura(ErroProcessoObter),
    Transporte(FalhaTransporte),
    UnidadeNaoPublicada,
    SituacaoNaoLida,
    ProcessoNaoAbertoNaUnidade,
    Link(ErroLinkIncluirDocumento),
    LinkNaoPublicado,
    Tela(FalhaDaTela),
    Tipo(ErroCriar),
    Formulario(ErroSubmissaoFormulario),
    NivelRestritoSemHipotese,
    SubmissaoNaoComprovada,
    ArvoreIncompleta,
    EfeitoIncerto,
}
impl ErroPreviaCriacao {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.subcodigo(),
            Self::Transporte(erro) => erro.subcodigo(),
            Self::UnidadeNaoPublicada => "unidade_da_sessao_nao_publicada",
            Self::SituacaoNaoLida => "situacao_do_processo_nao_lida_antes_de_criar_documento",
            Self::ProcessoNaoAbertoNaUnidade => "processo_nao_aberto_na_unidade_da_sessao",
            Self::Link(erro) => erro.subcodigo(),
            Self::LinkNaoPublicado => "incluir_documento_nao_publicado_na_barra",
            Self::Tela(FalhaDaTela::Transporte(erro)) => erro.subcodigo(),
            Self::Tela(FalhaDaTela::Redirecionada) => "tela_de_criacao_redirecionada",
            Self::Tela(FalhaDaTela::SessaoEncerrada) => "sessao_encerrada_rode_ies_login",
            Self::Tipo(erro) => erro.subcodigo(),
            Self::Formulario(erro) => erro.subcodigo(),
            Self::NivelRestritoSemHipotese => "nivel_de_documento_requer_hipotese_legal",
            Self::SubmissaoNaoComprovada => "submissao_do_documento_nao_observada",
            Self::ArvoreIncompleta => "arvore_de_documentos_incompleta",
            Self::EfeitoIncerto => "efeito_incerto_na_criacao_de_documento",
        }
    }
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Abertura(erro) => erro.status_http(),
            Self::Formulario(erro) => erro.status_http(),
            Self::Transporte(erro) | Self::Tela(FalhaDaTela::Transporte(erro)) => {
                erro.status_http()
            }
            _ => None,
        }
    }
}

/// Abertura do processo (GETs e, quando necessário, POST de pesquisa);
/// GET da lista de tipos e GET do cadastro direto. Zero POSTs de criação.
pub(crate) fn executar_criacao<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoCriacao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
    confirmar: bool,
) -> Result<PreviaCriacao, ErroPreviaCriacao> {
    use ErroPreviaCriacao as E;
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(E::Transporte(FalhaTransporte::OrigemNaoAutorizada))?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, true),
        indice,
        &mut rede,
    )
    .map_err(E::Abertura)?;
    let unidade = aberto.unidade.ok_or(E::UnidadeNaoPublicada)?;
    let abertas = siglas_abertas(&aberto.arvore.aberto_em).map_err(|_| E::SituacaoNaoLida)?;
    if !abertas.contains(&unidade) {
        return Err(E::ProcessoNaoAbertoNaUnidade);
    }
    let link = acoes.incluir.map_err(E::Link)?.ok_or(E::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao).map_err(E::Transporte)?;
    let (html_lista, _) = obter_tela(&mut rede, &cookie, link.as_str()).map_err(E::Tela)?;
    let tipos = tipos_oferecidos(&html_lista).map_err(E::Tipo)?;
    conferir_tipo(pedido.tipo, &tipos).map_err(E::Tipo)?;
    let url_do_cadastro = link_direto_do_tipo(
        &instalacao,
        &html_lista,
        &aberto.arvore.processo.id,
        pedido.tipo,
    )
    .map_err(E::Tipo)?;
    let (html_cadastro, mime_cadastro) =
        obter_tela(&mut rede, &cookie, &url_do_cadastro).map_err(E::Tela)?;
    conferir_cadastro(&html_cadastro, pedido.tipo, pedido.descricao, pedido.nivel)
        .map_err(E::Tipo)?;
    let mut feito = PreviaCriacao {
        nup: pedido.nup.as_str().to_owned(),
        tipo: pedido.tipo.to_owned(),
        unidade,
        requisicoes: rede.feitas(),
        confirmado: false,
        numero_sei: None,
    };
    if !confirmar {
        return Ok(feito);
    }
    // O cadastro restrito exige escolha de hipótese legal. Enquanto a
    // operação não resolver esse controle, apenas o nível público é seguro.
    if pedido.nivel.is_some_and(|nivel| nivel != "publico") {
        return Err(E::NivelRestritoSemHipotese);
    }
    conferir_cadastro(&html_cadastro, pedido.tipo, pedido.descricao, Some("publico"))
        .map_err(E::Tipo)?;
    if !html_cadastro.contains("document.getElementById('hdnFlagDocumentoCadastro').value = '2'") {
        return Err(E::SubmissaoNaoComprovada);
    }
    // Pasta fechada oculta documentos: impossível provar uma criação única.
    if aberto.arvore.pastas_nao_carregadas {
        return Err(E::ArvoreIncompleta);
    }
    let codificacao = codificacao_da_tela(&mime_cadastro).ok_or(E::SubmissaoNaoComprovada)?;
    let servida = TelaServida {
        url: &url_do_cadastro,
        html: &html_cadastro,
        codificacao,
    };
    let mut campos = vec![
        ("hdnFlagDocumentoCadastro", "2"),
        ("rdoNivelAcesso", "0"),
    ];
    if let Some(descricao) = pedido.descricao {
        campos.push(("txtDescricao", descricao));
    }
    let submissao = derivar_submissao(
        &CADASTRAR, &instalacao, &servida, &feito.unidade, &campos,
    )
    .map_err(E::Formulario)?;
    // O POST é consumido exatamente uma vez. Independentemente da resposta,
    // a conferência seguinte é uma nova leitura do processo, jamais retry.
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |post, cookie| {
        rede.post(post, cookie)
    });
    let depois = obter_processo(
        sessao,
        &leitura(pedido.nup, pedido.unidade, false),
        indice,
        &mut rede,
    )
    .map_err(|_| E::EfeitoIncerto)?;
    if depois.arvore.pastas_nao_carregadas
        || aberto.arvore.processo.id != depois.arvore.processo.id
    {
        return Err(E::EfeitoIncerto);
    }
    let numero = conferir_criacao(
        &aberto.arvore.documentos, &depois.arvore.documentos, pedido.tipo,
    )
    .map_err(|_| E::EfeitoIncerto)?;
    // O HTML/302 recebido não é seguido. Uma recusa ou falha no envio
    // permanece efeito incerto mesmo com a árvore aparentemente alterada.
    if !matches!(
        resposta,
        Ok(RespostaDoEnvio::Html(_) | RespostaDoEnvio::Redirecionamento(_))
    ) {
        return Err(E::EfeitoIncerto);
    }
    feito.confirmado = true;
    feito.numero_sei = Some(numero);
    feito.requisicoes = rede.feitas();
    Ok(feito)
}
