//! Fluxo do cancelamento: abre o documento, confere o formulário observado,
//! prévia sem POST, uma escrita e uma nova leitura (sem retries/redirects).
use super::erro::{ErroCancelar, FalhaDoCancelamento, VerificacaoDoCancelamento as Ver};
use super::ler_link_cancelar;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{Cancelamento, ProcessoNaArvore};
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::documento::NumeroDocumento;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
    consulta_do_controlador, derivar_submissao, enviar_submissao_via, localizar_formulario,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroDocumentoObter, ErroProcessoObter, PedidoDoAnexo, PedidoDoDocumento,
    SessaoDoProcesso, abrir_documento_na_arvore,
};
use crate::transporte::FalhaTransporte;
use crate::visao_do_controle::EnvioDoControle as _;

/// O navegador envia o botão sbmSalvar e o motivo (GO SEI 5.0/UFRJ 4.1).
const CANCELAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::DocumentoCancelar,
    formulario: "frmDocumentoCancelar",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmSalvar"),
    campos_ocultos_exigidos: &["hdnInfraTipoPagina"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["txaMotivo"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::CancelamentoDoDocumento,
    },
};

pub(crate) struct PedidoDoCancelamento<'a> {
    pub(crate) numero: &'a NumeroDocumento,
    pub(crate) processo: Option<&'a NumeroProcesso>,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) motivo: &'a str,
    pub(crate) confirmar: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResultadoDaVerificacao {
    Previa,
    Cancelado {
        falha_do_envio: Option<FalhaDoCancelamento>,
    },
}

impl ResultadoDaVerificacao {
    pub(crate) fn cancelado(&self) -> bool {
        matches!(self, Self::Cancelado { .. })
    }

    pub(crate) fn falha_do_envio(&self) -> Option<&FalhaDoCancelamento> {
        match self {
            Self::Previa => None,
            Self::Cancelado { falha_do_envio } => falha_do_envio.as_ref(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentoCancelado {
    pub(crate) numero: String,
    pub(crate) id_documento: String,
    pub(crate) processo: ProcessoNaArvore,
    pub(crate) unidade: String,
    pub(crate) motivo: String,
    pub(crate) resultado: ResultadoDaVerificacao,
    pub(crate) requisicoes: u8,
}

pub(crate) fn cancelar_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDoCancelamento<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<DocumentoCancelado, ErroCancelar> {
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(ErroCancelar::Abertura(
        ErroDocumentoObter::Processo(ErroProcessoObter::Transporte(
            FalhaTransporte::OrigemNaoAutorizada,
        )),
    ))?;
    let motivo = pedido.motivo;
    let mut rede = Contador::novo(envio);
    let leitura = PedidoDoDocumento {
        numero: pedido.numero,
        processo: pedido.processo,
        unidade: pedido.unidade,
        anexo: PedidoDoAnexo::default(),
    };
    let aberto = abrir_documento_na_arvore(sessao, &leitura, indice, &mut rede)
        .map_err(ErroCancelar::Abertura)?;
    if aberto.natureza.cancelamento.cancelado() {
        return Err(ErroCancelar::DocumentoJaCancelado);
    }
    let link = ler_link_cancelar(
        &instalacao,
        &aberto.html,
        &aberto.id_documento,
        &aberto.processo.id,
    )
    .map_err(ErroCancelar::Link)?
    .ok_or(ErroCancelar::CancelamentoNaoOferecido)?;
    let cookie = aberto.cookie.clone();
    let (html, mime) = obter_tela(&mut rede, &cookie, link.as_str()).map_err(|f| match f {
        FalhaDaTela::Transporte(falha) => ErroCancelar::Tela(falha),
        FalhaDaTela::Redirecionada => ErroCancelar::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
            ErroCancelar::SessaoEncerrada
        }
    })?;
    let codificacao = codificacao_da_tela(&mime).ok_or(ErroCancelar::CharsetNaoSuportado)?;
    let tela = TelaServida {
        url: link.as_str(),
        html: &html,
        codificacao,
    };
    conferir_formulario(
        &instalacao,
        &tela,
        &aberto.processo.id,
        &aberto.id_documento,
    )?;
    // A unidade é parte da escrita: sem ela publicada, nada é enviado.
    let unidade = aberto.unidade.ok_or(ErroCancelar::UnidadeNaoPublicada)?;
    let sub = derivar_submissao(
        &CANCELAR,
        &instalacao,
        &tela,
        Some(unidade.as_str()),
        &[("txaMotivo", motivo)],
    )
    .map_err(ErroCancelar::Formulario)?;
    let feito = DocumentoCancelado {
        numero: aberto.numero,
        id_documento: aberto.id_documento,
        processo: aberto.processo,
        unidade,
        motivo: motivo.to_owned(),
        resultado: ResultadoDaVerificacao::Previa,
        requisicoes: rede.feitas(),
    };
    if !pedido.confirmar {
        return Ok(feito);
    }
    let falha = enviar(&mut rede, sessao, sub, &feito)?;
    reconciliar(sessao, &leitura, indice, &mut rede, feito, falha)
}

/// Releitura independente de uma escrita já tentada; nunca tenta novo POST.
fn reconciliar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    leitura: &PedidoDoDocumento<'_>,
    indice: &IndiceLocal,
    rede: &mut Contador<'_, V>,
    mut feito: DocumentoCancelado,
    falha: Option<FalhaDoCancelamento>,
) -> Result<DocumentoCancelado, ErroCancelar> {
    // Sempre uma leitura nova depois de enviar. Até uma resposta 5xx pode
    // ter cancelado; a escrita irreversível jamais é repetida.
    let verificacao = match abrir_documento_na_arvore(sessao, leitura, indice, rede) {
        Ok(depois)
            if depois.id_documento == feito.id_documento
                && depois.processo.id == feito.processo.id =>
        {
            Ver::Lida {
                cancelamento: depois.natureza.cancelamento,
            }
        }
        Ok(_) => Ver::DocumentoDiferenteNaReleitura,
        Err(erro) => Ver::LeituraFalhou(erro),
    };
    feito.requisicoes = rede.feitas();
    match verificacao {
        Ver::Lida {
            cancelamento: Cancelamento::Cancelado { motivo: ref lido },
        } if lido == &feito.motivo => {
            feito.resultado = ResultadoDaVerificacao::Cancelado {
                falha_do_envio: falha,
            };
            Ok(feito)
        }
        Ver::Lida {
            cancelamento: Cancelamento::Ativo,
        } => Err(ErroCancelar::CancelamentoNaoAplicado {
            falha_do_envio: falha,
        }),
        outra @ (Ver::Lida { .. } | Ver::LeituraFalhou(_) | Ver::DocumentoDiferenteNaReleitura) => {
            Err(ErroCancelar::NaoConfirmado {
                falha_do_envio: falha,
                verificacao: Box::new(outra),
            })
        }
    }
}

/// Antes do envio, mesmo que `derivar_submissao` tenha validado a ação,
/// confere as duas identidades nela e exige um textarea editável único.
pub(super) fn conferir_formulario(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    processo: &str,
    documento: &str,
) -> Result<(), ErroCancelar> {
    use crate::arvore_processo::endereco::conferir_endereco_relativo_por;
    let dom = crate::tela_html::arvore(tela.html);
    let form = localizar_formulario(&dom, CANCELAR.formulario).map_err(ErroCancelar::Formulario)?;
    let action = form
        .atributo("action")
        .map(str::to_owned)
        .ok_or(ErroCancelar::FormularioDeOutroDocumento)?;
    conferir_endereco_relativo_por(
        instalacao,
        &action,
        CANCELAR.acao,
        "id_procedimento",
        processo,
    )
    .map_err(ErroCancelar::EnderecoDoProcessoRecusado)?;
    conferir_endereco_relativo_por(
        instalacao,
        &action,
        CANCELAR.acao,
        "id_documento",
        documento,
    )
    .map_err(ErroCancelar::EnderecoDoDocumentoRecusado)?;
    let mut motivos = 0;
    for tag in form.descendentes() {
        let nome = tag.atributo("name");
        if nome == Some("txaMotivo") {
            if tag.nome() != "textarea" || tag.presente("disabled") {
                return Err(ErroCancelar::MotivoNaoPublicadoNaTela);
            }
            motivos += 1;
        }
    }
    if motivos != 1 {
        return Err(ErroCancelar::MotivoNaoPublicadoNaTela);
    }
    Ok(())
}

/// O 302 aceito tem a ação, a identidade do processo/documento e o marcador
/// `atualizar_arvore=1` exatamente uma vez; Location nunca é seguido.
pub(super) fn localizacao_aceita(
    localizacao: &str,
    enviado: &str,
    id_proc: &str,
    id_doc: &str,
) -> bool {
    let Some(consulta) = consulta_do_controlador(localizacao, enviado) else {
        return false;
    };
    let Ok(consulta) = ConsultaDoControlador::ler(consulta) else {
        return false;
    };
    consulta.acao() == Ok(AcaoDoSei::ArvoreVisualizar.como_na_url())
        && consulta.unico("id_procedimento") == Ok(id_proc)
        && consulta.unico("id_documento") == Ok(id_doc)
        && consulta.unico("atualizar_arvore") == Ok("1")
}

fn enviar<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    submissao: SubmissaoDerivada,
    feito: &DocumentoCancelado,
) -> Result<Option<FalhaDoCancelamento>, ErroCancelar> {
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
        enviado = true;
        rede.post(s, c)
    });
    match resposta {
        Ok(RespostaDoEnvio::Redirecionamento(r)) => Ok((!r.localizacao().is_some_and(|loc| {
            localizacao_aceita(
                loc,
                r.url_do_envio(),
                &feito.processo.id,
                &feito.id_documento,
            )
        }))
        .then_some(FalhaDoCancelamento::RedirecionamentoNaoReconhecido)),
        Ok(RespostaDoEnvio::Html(_)) => Ok(Some(FalhaDoCancelamento::RespostaSemRedirecionamento)),
        Err(erro) if enviado => Ok(Some(FalhaDoCancelamento::Envio(erro))),
        Err(erro) => Err(ErroCancelar::Formulario(erro)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn destino_do_302_precisa_do_documento_e_do_marcador() {
        let post = "https://sei.exemplo.gov.br/sei/controlador.php?acao=documento_cancelar";
        assert!(localizacao_aceita(
            "controlador.php?acao=arvore_visualizar&id_procedimento=10&id_documento=21&atualizar_arvore=1",
            post,
            "10",
            "21"
        ));
        assert!(!localizacao_aceita(
            "controlador.php?acao=arvore_visualizar&id_procedimento=10&id_documento=99&atualizar_arvore=1",
            post,
            "10",
            "21"
        ));
        assert!(!localizacao_aceita(
            "controlador.php?acao=arvore_visualizar&id_procedimento=10&id_documento=21",
            post,
            "10",
            "21"
        ));
        assert!(!localizacao_aceita(
            "https://dominio-estranho.example/controlador.php?acao=arvore_visualizar&id_procedimento=10&id_documento=21&atualizar_arvore=1",
            post,
            "10",
            "21"
        ));
    }
}
