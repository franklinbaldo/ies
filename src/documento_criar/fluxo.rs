//! Prévia Web e confirmação de documento interno (#246).
//! O formulário é derivado da tela servida, sem executar JS nem repetir escrita.
use super::confirmacao::{FotoDaCriacao, postar_e_verificar};
use super::guarda_script::cadastro_oferece_salvar;
use super::{
    ErroCriar, EscolhaDoDocumento, cadastro_redirecionado, conferir_cadastro, conferir_tipo,
    escolha_do_tipo, filtro_de_todos, tipos_oferecidos,
};
use crate::arvore_processo::ErroLinkIncluirDocumento;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, EstadoDoProcesso, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio,
    TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::escrita::{
    Contador, FalhaDaTela, leitura, ler_cookie, obter_tela, siglas_abertas,
};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever, obter_processo,
};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};
use crate::visao_do_controle::EnvioDoControle as _;

// A forma de documento_gerar observada no Todos 323eb01 e em
// docs/como-o-sei-funciona/inclusao-documento-interno.md: submit() reescreve
// hdnFlagDocumentoCadastro para "2". Nenhum JavaScript é executado.
// Na UFRJ (SEI 4.1.5), escolher(id) só abre o cadastro do tipo:
// POST de navegação observado, sem inclusão de documento.
// Exibir todos somente quando a lista curta não mostrou o tipo. POST de
// leitura da lista, nunca de criação; exige o script observado da tela.
pub(super) const FILTRAR_TIPOS: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "documento_escolher_tipo",
    formulario: "frmDocumentoEscolherTipo",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnFiltroSerie"],
    campos_ocultos_reescritos: &["hdnFiltroSerie"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

pub(super) const ESCOLHER_TIPO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "documento_escolher_tipo",
    formulario: "frmDocumentoEscolherTipo",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnIdSerie"],
    campos_ocultos_reescritos: &["hdnIdSerie"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "docs/como-o-sei-funciona/inclusao-documento-interno.md: escolher(id) abre cadastro",
    },
};

pub(super) const CADASTRAR: OperacaoDeFormulario = OperacaoDeFormulario {
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

/// O resultado não permite representar uma confirmação sem número SEI.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EstadoCriacao {
    Previa,
    Criado { numero_sei: String },
}

pub(crate) struct ResultadoCriacao {
    pub(crate) nup: String,
    pub(crate) tipo: String,
    pub(crate) unidade: String,
    pub(crate) descricao: Option<String>,
    pub(crate) nivel: Option<String>,
    pub(crate) requisicoes: u8,
    pub(crate) estado: EstadoCriacao,
}

#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum ErroPreviaCriacao {
    /// O cookie privado da sessão não pôde ser lido.
    #[causa(interna)]
    Cookie(CookieIlegivel),
    #[causa(interna)]
    Abertura(ErroProcessoObter),
    OrigemNaoAutorizada,
    UnidadeNaoPublicada,
    SituacaoNaoLida,
    ProcessoNaoAbertoNaUnidade,
    #[causa(interna)]
    Link(ErroLinkIncluirDocumento),
    LinkNaoPublicado,
    #[causa(interna)]
    TelaTransporte(FalhaTransporte),
    TelaRedirecionada,
    #[causa(sessao_encerrada)]
    SessaoEncerrada,
    #[causa(interna)]
    Tipo(ErroCriar),
    #[causa(interna)]
    Formulario(ErroSubmissaoFormulario),
    NivelNaoInformado,
    NivelRestritoSemHipotese,
    SubmissaoNaoComprovada,
    ArvoreIncompleta,
    FotoInicialInconsistente,
    EfeitoIncerto,
    /// O POST pode ter produzido efeito mesmo com falha HTTP ou de rede;
    /// preservar sua causa sem autorizar uma segunda tentativa.
    EfeitoIncertoNoEnvio(ErroSubmissaoFormulario),
    /// O GET de reconciliação falhou após o POST; preservar a causa
    /// observada, sem inferir que o documento deixou de ser criado.
    EfeitoIncertoNaReleitura(ErroProcessoObter),
}
impl ErroPreviaCriacao {
    pub(crate) fn causa_envio(&self) -> Option<&ErroSubmissaoFormulario> {
        match self {
            Self::EfeitoIncertoNoEnvio(causa) => Some(causa),
            _ => None,
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        // A folha tipada preserva a classificação, mas o status HTTP
        // público inclui recusas da Infra (200) que o derive não marca.
        match self {
            Self::Abertura(erro) => erro.status_http(),
            Self::Formulario(erro) => erro.status_http(),
            Self::TelaTransporte(erro) => erro.status_http(),
            Self::EfeitoIncertoNoEnvio(causa) => causa.status_http(),
            Self::EfeitoIncertoNaReleitura(causa) => causa.status_http(),
            _ => None,
        }
    }
}

/// A falha da tela é traduzida para uma causa específica desta operação.
/// O transporte mantém a causa original; redirecionamento e login são fatos.
fn erro_da_tela(falha: FalhaDaTela) -> ErroPreviaCriacao {
    match falha {
        FalhaDaTela::Transporte(falha) => ErroPreviaCriacao::TelaTransporte(falha),
        FalhaDaTela::Redirecionada => ErroPreviaCriacao::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada => ErroPreviaCriacao::SessaoEncerrada,
    }
}

/// Pre-condicao independente da sessao: a operacao ainda so resolve nivel publico.
/// Uma unica regra atende tanto a fronteira CLI quanto o nucleo de escrita.
pub(crate) fn validar_nivel_de_confirmacao(nivel: Option<&str>) -> Result<(), ErroPreviaCriacao> {
    match nivel {
        Some("publico") => Ok(()),
        None => Err(ErroPreviaCriacao::NivelNaoInformado),
        _ => Err(ErroPreviaCriacao::NivelRestritoSemHipotese),
    }
}

/// Abertura do processo, seleção e cadastro; na confirmação, um único POST
/// seguido de leitura fresca. A prévia nunca envia POST criador.
#[allow(clippy::too_many_lines)] // Fluxo sequencial com uma única fronteira de escrita.
pub(crate) fn executar_criacao<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoCriacao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
    confirmar: bool,
) -> Result<ResultadoCriacao, ErroPreviaCriacao> {
    use ErroPreviaCriacao as E;
    // A operação já sabe que não pode criar sem nível explícito ou sem
    // hipótese legal resolvida. Não abrir sequer o processo: GET pode ter
    // efeito de recebimento e não serve a uma escrita que será recusada.
    if confirmar {
        validar_nivel_de_confirmacao(pedido.nivel)?;
    }
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(E::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, true),
        indice,
        &mut rede,
    )?;
    let unidade = aberto.unidade.ok_or(E::UnidadeNaoPublicada)?;
    let abertas = siglas_abertas(&aberto.arvore.aberto_em).map_err(|_| E::SituacaoNaoLida)?;
    if !abertas.contains(&unidade) {
        return Err(E::ProcessoNaoAbertoNaUnidade);
    }
    let link = acoes.incluir?.ok_or(E::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao)?;
    let (mut html_lista, mime_lista) =
        obter_tela(&mut rede, &cookie, link.as_str()).map_err(erro_da_tela)?;
    let mut tipos = tipos_oferecidos(&html_lista)?;
    // A lista inicial pode conter só os tipos recentes. Mesmo quando
    // o nome já aparece, a lista completa precisa confirmar sua unicidade:
    // outra série com o mesmo nome pode estar oculta pelo filtro.
    if filtro_de_todos(&html_lista)? {
        let codificacao = codificacao_da_tela(&mime_lista).ok_or(E::SubmissaoNaoComprovada)?;
        let servida = TelaServida {
            url: link.as_str(),
            html: &html_lista,
            codificacao,
        };
        let submissao = derivar_submissao(
            &FILTRAR_TIPOS,
            &instalacao,
            &servida,
            &unidade,
            &[("hdnFiltroSerie", "T")],
        )?;
        let resposta = enviar_submissao_via(submissao, sessao.cookie, |post, cookie| {
            rede.post(post, cookie)
        })?;
        // A evidência conhecida é o HTML da lista completa. Nunca seguir
        // 302 desconhecido nem repetir o POST da mudança de filtro.
        let RespostaDoEnvio::Html(completa) = resposta else {
            return Err(E::SubmissaoNaoComprovada);
        };
        html_lista = completa;
        tipos = tipos_oferecidos(&html_lista)?;
    }
    conferir_tipo(pedido.tipo, &tipos)?;
    let escolha = escolha_do_tipo(
        &instalacao,
        &html_lista,
        &aberto.arvore.processo.id,
        pedido.tipo,
    )?;
    let url_do_cadastro = match escolha {
        EscolhaDoDocumento::Link(url) => url,
        EscolhaDoDocumento::Formulario(id_serie) => {
            let codificacao = codificacao_da_tela(&mime_lista).ok_or(E::SubmissaoNaoComprovada)?;
            let servida = TelaServida {
                url: link.as_str(),
                html: &html_lista,
                codificacao,
            };
            let submissao = derivar_submissao(
                &ESCOLHER_TIPO,
                &instalacao,
                &servida,
                &unidade,
                &[("hdnIdSerie", id_serie.as_str())],
            )?;
            // Escolher o tipo só navega ao cadastro. 302 não é seguido
            // automaticamente; a Location assinada é validada antes do GET.
            let resposta = enviar_submissao_via(submissao, sessao.cookie, |post, cookie| {
                rede.post(post, cookie)
            })?;
            let RespostaDoEnvio::Redirecionamento(redirect) = resposta else {
                return Err(E::SubmissaoNaoComprovada);
            };
            cadastro_redirecionado(
                &instalacao,
                redirect.localizacao().ok_or(E::SubmissaoNaoComprovada)?,
                &aberto.arvore.processo.id,
            )?
        }
    };
    let (html_cadastro, mime_cadastro) =
        obter_tela(&mut rede, &cookie, &url_do_cadastro).map_err(erro_da_tela)?;
    conferir_cadastro(&html_cadastro, pedido.tipo, pedido.descricao, pedido.nivel)?;
    let previa = ResultadoCriacao {
        nup: pedido.nup.as_str().to_owned(),
        tipo: pedido.tipo.to_owned(),
        unidade,
        descricao: pedido.descricao.map(str::to_owned),
        nivel: pedido.nivel.map(str::to_owned),
        requisicoes: rede.feitas(),
        estado: EstadoCriacao::Previa,
    };
    if !confirmar {
        return Ok(previa);
    }
    // O cadastro restrito exige escolha de hipótese legal. Enquanto a
    // operação não resolver esse controle, apenas o nível público é seguro.
    conferir_cadastro(
        &html_cadastro,
        pedido.tipo,
        pedido.descricao,
        Some("publico"),
    )?;
    if !cadastro_oferece_salvar(&html_cadastro) {
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
    let mut campos = vec![("hdnFlagDocumentoCadastro", "2"), ("rdoNivelAcesso", "0")];
    if let Some(descricao) = pedido.descricao {
        campos.push(("txtDescricao", descricao));
    }
    let submissao = derivar_submissao(&CADASTRAR, &instalacao, &servida, &previa.unidade, &campos)?;
    // O POST é consumido exatamente uma vez. Independentemente da resposta,
    // a conferência seguinte é uma nova leitura do processo, jamais retry.
    let foto_antes = FotoDaCriacao::da_arvore(&aberto.arvore);
    let numero = postar_e_verificar(
        &mut rede,
        submissao,
        sessao.cookie,
        &foto_antes,
        pedido.tipo,
        crate::visao_do_controle::EnvioDoControle::post,
        |rede| {
            obter_processo(
                sessao,
                &leitura(pedido.nup, pedido.unidade, false),
                indice,
                rede,
            )
            .map(|depois| FotoDaCriacao::da_arvore(&depois.arvore))
        },
    )?;
    Ok(ResultadoCriacao {
        requisicoes: rede.feitas(),
        estado: EstadoCriacao::Criado { numero_sei: numero },
        ..previa
    })
}

#[cfg(test)]
#[path = "tests_fluxo.rs"]
mod tests;

#[cfg(test)]
#[path = "tests_confirmacao.rs"]
mod tests_confirmacao;

#[cfg(test)]
#[path = "tests_integracao.rs"]
mod tests_integracao;
