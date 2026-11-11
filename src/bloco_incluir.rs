//! `ies bloco incluir BLOCO --documento DOC` (#381; contracts/cli/bloco-incluir.md):
//! prévia e confirmação causal de documento em bloco de assinatura. Abre o documento
//! até a árvore, segue o `bloco_escolher` publicado na barra do documento e
//! confere no `frmBlocoEscolher` que o bloco e o documento são oferecidos;
//! o pertencimento anterior vem do detalhe do bloco, a mesma leitura de
//! `ies bloco N`. Confirmação: um POST e releitura independente (BDD-381-02).
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{ErroBlocoNaBarra, ler_bloco_escolher_da_barra};
use crate::bloco::TipoBloco;
use crate::bloco_cli::{FalhaLeitura, Leitura, ler_tipado_via};
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::documento::NumeroDocumento;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, consulta_do_controlador,
    derivar_submissao, enviar_submissao_via,
};
use crate::html_atributos::valor_decodificado;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_concluir::escrita::{
    Contador, CookieIlegivel, FalhaDaTela, ler_cookie, obter_tela,
};
use crate::processo_obter::{
    DocumentoNaArvoreAberto, EnvioDoProcesso, ErroDocumentoObter, PedidoDoAnexo, PedidoDoDocumento,
    SessaoDoProcesso, abrir_documento_na_arvore,
};
use crate::transporte::FalhaTransporte;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::EnvioDoControle as _;

/// Blocos e documentos oferecidos pelo `frmBlocoEscolher`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TelaDeEscolha {
    /// Números dos blocos de `selBloco` (sem a opção vazia).
    pub(crate) blocos: Vec<String>,
    /// Cada caixa preserva o vínculo entre nome do controle, ID e número SEI.
    pub(crate) documentos: Vec<DocumentoOferecido>,
}

/// Controle efetivamente oferecido pelo formulário, não um índice presumido.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DocumentoOferecido {
    nome: String,
    id: String,
    numero: String,
}

/// Prévia da inclusão: o que foi observado, sem gravar.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PreviaDocumental {
    pub(crate) bloco: String,
    pub(crate) documento: String,
    pub(crate) requisicoes: usize,
    pub(crate) desfecho: DesfechoDaInclusao,
}

/// Desfecho observado da inclusão. Cada variante carrega só o que existe nela:
/// o pertencimento antes e depois, e a solicitação, são derivados daqui.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DesfechoDaInclusao {
    /// Prévia sem POST: o detalhe do bloco é completo e não contém o documento.
    Previa,
    /// Já presente no detalhe: nenhuma escrita, com ou sem o pedido de confirmação.
    JaEstava { confirmacao_pedida: bool },
    /// POST aceito e releitura independente confirmou o documento no bloco.
    Incluido,
}

impl DesfechoDaInclusao {
    /// A solicitação do usuário (`--confirmar`), não prova autônoma de efeito.
    pub(crate) fn confirmado(&self) -> bool {
        match self {
            Self::Previa => false,
            Self::JaEstava { confirmacao_pedida } => *confirmacao_pedida,
            Self::Incluido => true,
        }
    }

    pub(crate) fn alterado(&self) -> bool {
        matches!(self, Self::Incluido)
    }

    /// Pertencimento lido no detalhe antes de qualquer escrita.
    pub(crate) fn presente_antes(&self) -> bool {
        match self {
            Self::Previa | Self::Incluido => false,
            Self::JaEstava { .. } => true,
        }
    }

    /// Sem escrita, o estado depois é o próprio estado lido antes.
    pub(crate) fn presente_depois(&self) -> bool {
        self.presente_antes() || self.alterado()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroIncluir {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[from] crate::html_atributos::ErroAtributoHtml),
    Abertura(#[from] ErroDocumentoObter),
    Barra(#[from] ErroBlocoNaBarra),
    /// A barra do documento não oferece "Incluir em Bloco de Assinatura".
    AcaoNaoOferecidaNaBarra,
    Cookie(#[from] CookieIlegivel),
    Tela(#[from] FalhaDaTela),
    /// `selBloco` não oferece o bloco pedido.
    BlocoNaoOferecido,
    /// A tela de escolha não lista o documento pedido.
    DocumentoForaDaTela,
    LeituraDoBloco(#[from] FalhaLeitura),
    Transporte(#[from] FalhaTransporte),
    Formulario(#[from] crate::formulario_web::ErroSubmissaoFormulario),
    CharsetNaoSuportado,
    /// O detalhe do bloco não lista todos os seus itens (`lidos` de `declarados`)
    /// e não contém o documento: o pertencimento não pode ser afirmado.
    DetalheIncompletoSemDocumento {
        lidos: usize,
        declarados: usize,
    },
    NaoConfirmado,
    CaixasDeDocumentosAmbiguas,
    CaixasDoDocumentoPedido,
    FormularioDeEscolherBlocoAusente,
    FormulariosDeEscolherBlocoRepetidos,
    SelecaoDeBlocoAusente,
    SelecoesDeBlocoRepetidas,
    TelaDeEscolherBlocoForaDaFormaObservada,
}

/// Lê o `frmBlocoEscolher`: um formulário, um `selBloco`.
pub(crate) fn ler_tela_de_escolha(
    html: &crate::tela_html::HtmlServido,
) -> Result<TelaDeEscolha, ErroIncluir> {
    let fora = || ErroIncluir::TelaDeEscolherBlocoForaDaFormaObservada;
    let dom = crate::tela_html::interpretar(html);
    let parser = dom.parser();
    let tags = |seletor: &str| -> Vec<&tl::HTMLTag<'_>> {
        dom.query_selector(seletor)
            .into_iter()
            .flatten()
            .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
            .collect()
    };
    let formularios = tags("form#frmBlocoEscolher");
    match formularios.len() {
        0 => return Err(ErroIncluir::FormularioDeEscolherBlocoAusente),
        1 => {}
        _ => {
            return Err(ErroIncluir::FormulariosDeEscolherBlocoRepetidos);
        }
    }
    let selecoes = tags("select#selBloco");
    let selecao = match selecoes.as_slice() {
        [] => return Err(ErroIncluir::SelecaoDeBlocoAusente),
        [selecao] => *selecao,
        _ => return Err(ErroIncluir::SelecoesDeBlocoRepetidas),
    };
    let mut blocos = Vec::new();
    for filho in selecao.children().all(parser) {
        let Some(opcao) = filho.as_tag().filter(|t| t.name() == "option") else {
            continue;
        };
        let valor = valor_decodificado(opcao, "value")?.ok_or_else(fora)?;
        if valor == "null" {
            continue;
        }
        if valor.is_empty() || !valor.bytes().all(|b| b.is_ascii_digit()) {
            return Err(fora());
        }
        blocos.push(valor);
    }
    let mut documentos: Vec<DocumentoOferecido> = Vec::new();
    for caixa in tags("input") {
        let Some(nome) = valor_decodificado(caixa, "name")? else {
            continue;
        };
        let Some(indice) = nome.strip_prefix("chkDocumentosItem") else {
            continue;
        };
        if indice.is_empty()
            || !indice.bytes().all(|b| b.is_ascii_digit())
            || valor_decodificado(caixa, "id")?.as_deref() != Some(nome.as_str())
            || valor_decodificado(caixa, "type")?.as_deref() != Some("checkbox")
        {
            return Err(fora());
        }
        let id = valor_decodificado(caixa, "value")?.ok_or_else(fora)?;
        let numero = valor_decodificado(caixa, "title")?.ok_or_else(fora)?;
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) || numero.is_empty() {
            return Err(fora());
        }
        if documentos
            .iter()
            .any(|d| d.nome == nome || d.id == id || d.numero == numero)
        {
            return Err(ErroIncluir::CaixasDeDocumentosAmbiguas);
        }
        documentos.push(DocumentoOferecido { nome, id, numero });
    }
    Ok(TelaDeEscolha { blocos, documentos })
}

/// Identidade pública e intenção do comando, sem transporte ou estado duplicado.
pub(crate) struct PedidoDaInclusao<'a> {
    pub(crate) bloco: &'a str,
    pub(crate) documento: &'a NumeroDocumento,
    pub(crate) confirmar: bool,
}

/// Prévia ou confirmação: formulário fresco, no máximo um POST, releitura.
pub(crate) fn prever_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    referencias: &ReferenciasDaSessao,
    unidade: Option<&str>,
    pedido: &PedidoDaInclusao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<PreviaDocumental, ErroIncluir> {
    let bloco = pedido.bloco;
    let documento = pedido.documento;
    let confirmar = pedido.confirmar;
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(ErroIncluir::Transporte(
        FalhaTransporte::OrigemNaoAutorizada,
    ))?;
    let mut rede = Contador::novo(envio);
    let pedido = PedidoDoDocumento {
        numero: documento,
        unidade,
        processo: None,
        anexo: PedidoDoAnexo::default(),
    };
    let aberto = abrir_documento_na_arvore(sessao, &pedido, indice, &mut rede)?;
    // A leitura independente do detalhe precede a exigência de uma ação
    // oferecida. Um documento já comprovado no bloco não requer formulário,
    // nem GET da tela de inclusão, nem POST de escrita.
    let lido = ler_tipado_via(
        referencias,
        unidade,
        TipoBloco::Assinatura,
        Some(bloco),
        None,
        &mut rede,
    )?;
    let Leitura::Detalhe(detalhe) = lido.leitura else {
        unreachable!("com número, a leitura é o detalhe do bloco");
    };
    let presente = detalhe
        .itens
        .iter()
        .any(|item| item.documento.as_deref() == Some(aberto.numero.as_str()));
    let completo = detalhe.itens.len() == detalhe.total_declarado;
    if !presente && !completo {
        return Err(ErroIncluir::DetalheIncompletoSemDocumento {
            lidos: detalhe.itens.len(),
            declarados: detalhe.total_declarado,
        });
    }
    if presente {
        return Ok(PreviaDocumental {
            bloco: bloco.to_owned(),
            documento: aberto.numero,
            requisicoes: usize::from(rede.feitas()),
            desfecho: DesfechoDaInclusao::JaEstava {
                confirmacao_pedida: confirmar,
            },
        });
    }
    let link = ler_bloco_escolher_da_barra(
        &instalacao,
        &aberto.html,
        &aberto.id_documento,
        &aberto.processo.id,
    )?
    .ok_or(ErroIncluir::AcaoNaoOferecidaNaBarra)?;
    let cookie = ler_cookie(sessao)?;
    let (html, mime) = obter_tela(&mut rede, &cookie, link.as_str())?;
    let tela = ler_tela_de_escolha(&html)?;
    let caixa = caixa_do_documento(&tela, bloco, &aberto)?;
    let desfecho = if confirmar {
        confirmar_documento(
            sessao,
            referencias,
            unidade,
            bloco,
            &aberto,
            &TelaDaInclusao {
                url: link.as_str(),
                html: &html,
                mime: &mime,
                nome_da_caixa: &caixa.nome,
            },
            &mut rede,
        )?;
        DesfechoDaInclusao::Incluido
    } else {
        DesfechoDaInclusao::Previa
    };
    Ok(PreviaDocumental {
        bloco: bloco.to_owned(),
        documento: aberto.numero,
        requisicoes: usize::from(rede.feitas()),
        desfecho,
    })
}

/// Confere na tela de escolha o bloco pedido e a única caixa do documento.
fn caixa_do_documento<'t>(
    tela: &'t TelaDeEscolha,
    bloco: &str,
    aberto: &DocumentoNaArvoreAberto,
) -> Result<&'t DocumentoOferecido, ErroIncluir> {
    if !tela.blocos.iter().any(|b| b == bloco) {
        return Err(ErroIncluir::BlocoNaoOferecido);
    }
    let mut caixas = tela.documentos.iter().filter(|d| d.numero == aberto.numero);
    let caixa = caixas.next().ok_or(ErroIncluir::DocumentoForaDaTela)?;
    if caixas.next().is_some() {
        return Err(ErroIncluir::CaixasDoDocumentoPedido);
    }
    if caixa.id != aberto.id_documento {
        return Err(ErroIncluir::DocumentoForaDaTela);
    }
    Ok(caixa)
}

/// A tela fresca da oferta de inclusão, sem persistir HTML assinado.
struct TelaDaInclusao<'a> {
    url: &'a str,
    html: &'a crate::tela_html::HtmlServido,
    mime: &'a str,
    nome_da_caixa: &'a str,
}

/// Envio causal único e uma leitura independente. Ambos são necessários
/// para declarar alteração, mesmo se a resposta 302 foi observada.
fn confirmar_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    referencias: &ReferenciasDaSessao,
    unidade: Option<&str>,
    bloco: &str,
    aberto: &DocumentoNaArvoreAberto,
    tela: &TelaDaInclusao<'_>,
    rede: &mut Contador<'_, V>,
) -> Result<(), ErroIncluir> {
    let instalacao =
        Instalacao::interpretar(sessao.origem).ok_or(FalhaTransporte::OrigemNaoAutorizada)?;
    let codificacao = codificacao_da_tela(tela.mime).ok_or(ErroIncluir::CharsetNaoSuportado)?;
    let servida = TelaServida {
        url: tela.url,
        html: tela.html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &INCLUIR_DOCUMENTO,
        &instalacao,
        &servida,
        None,
        &[
            ("selBloco", bloco),
            (tela.nome_da_caixa, &aberto.id_documento),
            ("hdnDocumentosItensSelecionados", &aberto.id_documento),
        ],
    )?;
    // Submissão consumida, sem possibilidade de segundo POST.
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |sub, cookie| {
        rede.post(sub, cookie)
    });
    let aceita = match resposta {
        Ok(RespostaDoEnvio::Redirecionamento(r)) => redirecionamento_da_inclusao(
            r.localizacao(),
            r.url_do_envio(),
            &aberto.processo.id,
            &aberto.id_documento,
        ),
        _ => false,
    };
    let relido = ler_tipado_via(
        referencias,
        unidade,
        TipoBloco::Assinatura,
        Some(bloco),
        None,
        rede,
    );
    let conferido = matches!(
        relido,
        Ok(crate::bloco_cli::Lido { leitura: Leitura::Detalhe(ref detalhe), .. })
            if detalhe.itens.iter().any(|item| {
                item.documento.as_deref() == Some(aberto.numero.as_str())
            })
    );
    if !aceita || !conferido {
        return Err(ErroIncluir::NaoConfirmado);
    }
    Ok(())
}

const INCLUIR_DOCUMENTO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::BlocoEscolher,
    formulario: "frmBlocoEscolher",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmIncluir"),
    campos_ocultos_exigidos: &["hdnDocumentosItens", "hdnDocumentosItensSelecionados"],
    campos_ocultos_reescritos: &["hdnDocumentosItensSelecionados"],
    campos_usuario: &["selBloco", "chkDocumentosItem*"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::DocumentoIncluidoNoBloco,
    },
};

/// O 302 é apenas aceitação causal da inclusão quando vinculado à mesma
/// identidade e à ação publicada, sem seguir redirecionamento.
fn redirecionamento_da_inclusao(
    location: Option<&str>,
    url_do_envio: &str,
    id_procedimento: &str,
    id_documento: &str,
) -> bool {
    let Some(consulta) = location.and_then(|loc| consulta_do_controlador(loc, url_do_envio)) else {
        return false;
    };
    ConsultaDoControlador::ler(consulta).is_ok_and(|consulta| {
        consulta.acao() == Ok(AcaoDoSei::BlocoEscolher)
            && consulta.unico("id_procedimento") == Ok(id_procedimento)
            && consulta.unico("id_documento") == Ok(id_documento)
    })
}

#[cfg(test)]
#[path = "bloco_incluir/tests.rs"]
mod tests;

#[cfg(test)]
mod acervo;
