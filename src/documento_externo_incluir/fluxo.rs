//! Prévia Web do documento externo PDF (#332): lê lista e cadastro recarregado.
//! A única escrita de documento segue desabilitada até upload + cadastro + releitura.
use super::{
    PdfLocal,
    cadastro::{CadastroExterno, observar_cadastro},
    confirmacao::FotoProcesso,
    escolha::{EscolhaExterna, escolher_externo, identificar_tipo},
};
use crate::arvore_processo::endereco::conferir_endereco_com_identidade_repetida;
use crate::formulario_web::{
    CodificacaoDaTela, ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario,
    ErroSubmissaoFormulario, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    derivar_submissao, enviar_submissao_via,
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
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};
use crate::visao_do_controle::EnvioDoControle as _;

const ESCOLHER_EXTERNO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "documento_escolher_tipo",
    formulario: "frmDocumentoEscolherTipo",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnIdSerie"],
    campos_ocultos_reescritos: &["hdnIdSerie"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "Todos tests/test_documento_juntar_010.py: escolher(-1) navega ao cadastro",
    },
};

const RECARREGAR_TIPO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "documento_receber",
    formulario: "frmDocumentoCadastro",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnIdSerie", "hdnFlagDocumentoCadastro"],
    campos_ocultos_reescritos: &["hdnIdSerie"],
    campos_usuario: &["selSerie"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::ConsultaSemEfeitoObservado {
        evidencia: "Todos documento_externo.py: recarga tipo com flag 1 não cadastra documento",
    },
};

pub(crate) struct PreviaExterna {
    pub(crate) unidade: String,
    pub(crate) id_tipo: String,
    pub(crate) limite_pdf_bytes: u64,
    pub(crate) campos_oferecidos: Vec<String>,
}

/// Material exato da mesma navegação que ofereceu a prévia ao usuário.
/// URLs assinadas e HTML não são expostos na apresentação nem em Debug.
pub(crate) struct CadastroPreparado {
    pub(crate) previa: PreviaExterna,
    pub(crate) cadastro: CadastroExterno,
    pub(crate) url_cadastro: String,
    pub(crate) html_cadastro: String,
    pub(crate) codificacao: CodificacaoDaTela,
    pub(crate) foto_antes: FotoProcesso,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroPreviaExterna {
    /// O cookie privado da sessão não pôde ser lido.
    Cookie(CookieIlegivel),
    Abertura(ErroProcessoObter),
    Transporte(FalhaTransporte),
    UnidadeNaoPublicada,
    ProcessoNaoAbertoNaUnidade,
    LinkNaoPublicado,
    LinkNaoConferido,
    TelaRedirecionada,
    SessaoEncerrada,
    EscolhaNaoConferida,
    TipoNaoOferecido,
    TipoNaoRecarregado,
    CadastroNaoConferido,
    Formulario(ErroSubmissaoFormulario),
    CodificacaoNaoPublicada,
}
impl ErroPreviaExterna {
    pub(crate) fn causa(&self) -> &'static str {
        match self {
            Self::Abertura(_) => "abertura_do_processo",
            Self::Transporte(_) | Self::Cookie(_) => "transporte",
            Self::UnidadeNaoPublicada => "unidade_nao_publicada",
            Self::ProcessoNaoAbertoNaUnidade => "processo_fora_da_unidade",
            Self::LinkNaoPublicado => "incluir_documento_nao_oferecido",
            Self::LinkNaoConferido => "link_externo_nao_conferido",
            Self::TelaRedirecionada => "tela_redirecionada",
            Self::SessaoEncerrada => "sessao_encerrada",
            Self::EscolhaNaoConferida => "escolha_externa_nao_observada",
            Self::TipoNaoOferecido => "tipo_externo_nao_oferecido",
            Self::TipoNaoRecarregado => "tipo_externo_nao_recarregado",
            Self::CadastroNaoConferido => "cadastro_externo_nao_conferido",
            Self::Formulario(_) => "formulario_externo_nao_submetido",
            Self::CodificacaoNaoPublicada => "codificacao_nao_publicada",
        }
    }
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Abertura(e) => e.status_http(),
            Self::Transporte(e) => e.status_http(),
            Self::Formulario(e) => e.status_http(),
            _ => None,
        }
    }
}
fn da_tela(erro: FalhaDaTela) -> ErroPreviaExterna {
    match erro {
        FalhaDaTela::Transporte(e) => ErroPreviaExterna::Transporte(e),
        FalhaDaTela::Redirecionada => ErroPreviaExterna::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
            ErroPreviaExterna::SessaoEncerrada
        }
    }
}

fn conferir_destino(
    instalacao: &Instalacao,
    location: &str,
    id_procedimento: &str,
) -> Result<String, ErroPreviaExterna> {
    let base = format!("{}/", instalacao.url_base());
    let relativo = location.strip_prefix(&base).unwrap_or(location);
    // SEI 4.0.9 (UnB) repete id_procedimento no link Externo da lista.
    conferir_endereco_com_identidade_repetida(
        instalacao,
        relativo,
        "documento_receber",
        id_procedimento,
    )
    .map_err(|_| ErroPreviaExterna::LinkNaoConferido)
}

/// Lê a árvore, escolhe Externo, recarrega o tipo e confere o cadastro do PDF.
/// Em nenhuma condição envia upload ou cadastro definitivo.
pub(crate) fn executar_previa<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade_pedida: Option<&str>,
    tipo: &str,
    pdf: &PdfLocal,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<PreviaExterna, ErroPreviaExterna> {
    preparar_previa(sessao, nup, unidade_pedida, tipo, pdf, indice, envio)
        .map(|preparo| preparo.previa)
}

#[allow(clippy::too_many_lines)] // Navegação Web existente, agora também conserva o cadastro.
pub(crate) fn preparar_previa<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade_pedida: Option<&str>,
    tipo: &str,
    pdf: &PdfLocal,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<CadastroPreparado, ErroPreviaExterna> {
    use ErroPreviaExterna as E;
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(E::Transporte(FalhaTransporte::OrigemNaoAutorizada))?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(nup, unidade_pedida, true),
        indice,
        &mut rede,
    )
    .map_err(E::Abertura)?;
    let unidade = aberto.unidade.ok_or(E::UnidadeNaoPublicada)?;
    let abertas =
        siglas_abertas(&aberto.arvore.aberto_em).map_err(|_| E::ProcessoNaoAbertoNaUnidade)?;
    if !abertas.contains(&unidade) {
        return Err(E::ProcessoNaoAbertoNaUnidade);
    }
    let link = acoes
        .incluir
        .map_err(|_| E::LinkNaoConferido)?
        .ok_or(E::LinkNaoPublicado)?;
    let cookie = ler_cookie(sessao)?;
    let (lista, mime_lista) = obter_tela(&mut rede, &cookie, link.as_str()).map_err(da_tela)?;
    let id_procedimento = &aberto.arvore.processo.id;
    let destino = match escolher_externo(&lista).map_err(|_| E::EscolhaNaoConferida)? {
        EscolhaExterna::Url(relativo) => conferir_destino(&instalacao, &relativo, id_procedimento)?,
        EscolhaExterna::Formulario => {
            let codificacao = codificacao_da_tela(&mime_lista).ok_or(E::CodificacaoNaoPublicada)?;
            let servida = TelaServida {
                url: link.as_str(),
                html: &lista,
                codificacao,
            };
            let submissao = derivar_submissao(
                &ESCOLHER_EXTERNO,
                &instalacao,
                &servida,
                &unidade,
                &[("hdnIdSerie", "-1")],
            )
            .map_err(E::Formulario)?;
            let resposta = enviar_submissao_via(submissao, sessao.cookie, |post, cookie| {
                rede.post(post, cookie)
            })
            .map_err(E::Formulario)?;
            let RespostaDoEnvio::Redirecionamento(redirecionado) = resposta else {
                return Err(E::EscolhaNaoConferida);
            };
            conferir_destino(
                &instalacao,
                redirecionado.localizacao().ok_or(E::LinkNaoConferido)?,
                id_procedimento,
            )?
        }
    };
    let (cadastro, mime_cadastro) = obter_tela(&mut rede, &cookie, &destino).map_err(da_tela)?;
    let id_tipo = identificar_tipo(&cadastro, tipo).map_err(|_| E::TipoNaoOferecido)?;
    let codificacao = codificacao_da_tela(&mime_cadastro).ok_or(E::CodificacaoNaoPublicada)?;
    let servida = TelaServida {
        url: &destino,
        html: &cadastro,
        codificacao,
    };
    let submissao = derivar_submissao(
        &RECARREGAR_TIPO,
        &instalacao,
        &servida,
        &unidade,
        &[
            ("selSerie", id_tipo.as_str()),
            ("hdnIdSerie", id_tipo.as_str()),
        ],
    )
    .map_err(E::Formulario)?;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |post, cookie| {
        rede.post(post, cookie)
    })
    .map_err(E::Formulario)?;
    let RespostaDoEnvio::Html(recarregado) = resposta else {
        return Err(E::TipoNaoRecarregado);
    };
    let observado = observar_cadastro(&recarregado, tipo, pdf.tamanho_bytes)
        .map_err(|_| E::CadastroNaoConferido)?;
    if observado.id_tipo != id_tipo {
        return Err(E::TipoNaoRecarregado);
    }
    let foto_antes = FotoProcesso::da_arvore(&aberto.arvore);
    let previa = PreviaExterna {
        unidade,
        id_tipo: observado.id_tipo.clone(),
        limite_pdf_bytes: observado.limite_pdf_bytes,
        campos_oferecidos: observado.campos_oferecidos.clone(),
    };
    Ok(CadastroPreparado {
        previa,
        cadastro: observado,
        url_cadastro: destino,
        html_cadastro: recarregado,
        codificacao,
        foto_antes,
    })
}

#[cfg(test)]
#[path = "tests_fluxo.rs"]
mod tests;

impl From<CookieIlegivel> for ErroPreviaExterna {
    fn from(erro: CookieIlegivel) -> Self {
        Self::Cookie(erro)
    }
}
