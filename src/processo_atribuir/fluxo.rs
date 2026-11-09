//! Prévia Web injetável da atribuição de processo: abertura, unidade,
//! atribuição publicada na árvore, formulário da barra e payload local.
//! A CLI mantém --confirmar fechado até haver validação live em treinamento.
use crate::arvore_processo::{AbertoEm, ErroAtribuirNaBarra};
use crate::formulario_web::{
    ErroSubmissaoFormulario, Redirecionamento302, RespostaDoEnvio, consulta_do_controlador,
    enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_atribuir::{ErroAtribuir, preparar_submissao};
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever, obter_processo,
};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};
use crate::visao_do_controle::EnvioDoControle as _;

pub(crate) struct PedidoDaAtribuicao<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) usuario: Option<&'a str>,
    pub(crate) confirmar: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDaAtribuicao {
    Previa,
    AlteracaoConfirmada,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoAtribuido {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) unidade: String,
    pub(crate) usuario_atual: String,
    pub(crate) usuario_pedido: Option<String>,
    pub(crate) valor_da_opcao: String,
    pub(crate) estado: EstadoDaAtribuicao,
    pub(crate) requisicoes: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroProcessoAtribuir {
    #[causa(interna)]
    Abertura(ErroProcessoObter),
    UnidadeDaSessaoNaoPublicada,
    #[causa(nome = "situacao_da_atribuicao_nao_observada")]
    SituacaoDaAtribuicaoNaoObservada {
        unidade: String,
        aberto_em: Option<Vec<String>>,
        motivo: Option<&'static str>,
        requisicoes: u8,
    },
    AtribuicaoNaoOferecidaNaBarra,
    #[causa(interna)]
    Barra(ErroAtribuirNaBarra),
    #[causa(interna)]
    Cookie(FalhaTransporte),
    #[causa(interna)]
    Tela(FalhaTransporte),
    TelaDaAtribuicaoRedirecionada,
    SessaoEncerradaNaAtribuicao,
    OrigemNaoAutorizada,
    #[causa(nome = "escrita_de_atribuicao_ainda_nao_verificada")]
    EscritaAindaNaoVerificada,
    #[causa(interna)]
    Preparacao(ErroAtribuir),
    #[causa(interna)]
    Formulario(ErroSubmissaoFormulario),
    #[causa(nome = "efeito_incerto_na_atribuicao")]
    EfeitoIncertoNaAtribuicao {
        falha_do_envio: Option<Box<ErroSubmissaoFormulario>>,
        leitura: Option<Box<ErroProcessoObter>>,
        atribuicao_relida: Option<String>,
        requisicoes: u8,
    },
}

impl ErroProcessoAtribuir {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Preparacao(erro) => erro.subcodigo(),
            Self::Barra(erro) => (*erro).subcodigo(),
            _ => crate::erro_publico::folha(self),
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Formulario(erro) => erro.status_http(),
            Self::EfeitoIncertoNaAtribuicao {
                falha_do_envio,
                leitura,
                ..
            } => falha_do_envio
                .as_deref()
                .and_then(ErroSubmissaoFormulario::status_http)
                // O POST pode ter sido aceito ou falhado sem resposta,
                // enquanto a leitura de prova recebeu recusa HTTP.
                // Não apagar o status e a mensagem dessa última falha.
                .or_else(|| leitura.as_deref().and_then(ErroProcessoObter::status_http)),
            _ => crate::erro_publico::Causa::status_http(self),
        }
    }

    pub(crate) fn saida(&self) -> u8 {
        if matches!(self, Self::EfeitoIncertoNaAtribuicao { .. }) {
            3
        } else {
            1
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.mensagem(),
            Self::UnidadeDaSessaoNaoPublicada => "a abertura não publicou a unidade da sessão",
            Self::SituacaoDaAtribuicaoNaoObservada { .. } => {
                "a árvore não publicou a atribuição na unidade da sessão"
            }
            Self::AtribuicaoNaoOferecidaNaBarra => {
                "a barra do processo não oferece atribuição nesta unidade"
            }
            Self::Barra(_) => "o link de atribuição não foi publicado na forma observada",
            Self::Cookie(_) => "o arquivo privado não forneceu um cookie válido",
            Self::Tela(_) => "a tela de atribuição não foi servida",
            Self::TelaDaAtribuicaoRedirecionada => {
                "a tela de atribuição redirecionou; nenhum formulário foi enviado"
            }
            Self::SessaoEncerradaNaAtribuicao => {
                "a sessão encerrou antes da atribuição; rode ies login"
            }
            Self::OrigemNaoAutorizada => "a origem da sessão não é autorizada",
            Self::EscritaAindaNaoVerificada => {
                "a confirmação não está habilitada nesta origem; execute sem --confirmar para consultar a prévia"
            }
            Self::Preparacao(ErroAtribuir::AtribuicaoJaAplicada) => {
                "a atribuição solicitada já está aplicada"
            }
            Self::Preparacao(_) => "o formulário de atribuição não corresponde ao observado",
            Self::Formulario(_) => "a submissão da atribuição não pôde ser enviada",
            Self::EfeitoIncertoNaAtribuicao { .. } => {
                "o efeito desta tentativa não foi comprovado; não repita a confirmação; consulte o processo"
            }
        }
    }
}

/// A primeira requisição só ocorre depois da conferência da trava de escrita.
pub(crate) fn executar_previa<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaAtribuicao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoAtribuido, ErroProcessoAtribuir> {
    use ErroProcessoAtribuir as E;
    let mut rede = Contador::novo(envio);
    let abertura = leitura(pedido.nup, pedido.unidade, true);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &abertura, indice, &mut rede).map_err(E::Abertura)?;
    let unidade = aberto
        .unidade
        .as_deref()
        .ok_or(E::UnidadeDaSessaoNaoPublicada)?;
    let atual = atribuicao_observada(&aberto.arvore.aberto_em, unidade).ok_or_else(|| {
        diagnostico_unidade_nao_observada(&aberto.arvore.aberto_em, unidade, rede.feitas())
    })?;
    // A árvore já informa quando a operação não alteraria nada.
    // Rejeitar antes de abrir o formulário evita um GET desnecessário,
    // inclusive nas confirmações injetáveis, sem mudar o estado remoto.
    let ja_aplicada = match pedido.usuario {
        Some(login) => !login.is_empty() && login == atual,
        None => atual.is_empty(),
    };
    if ja_aplicada {
        return Err(E::Preparacao(ErroAtribuir::AtribuicaoJaAplicada));
    }
    let link = acoes
        .atribuir
        .map_err(E::Barra)?
        .ok_or(E::AtribuicaoNaoOferecidaNaBarra)?;
    let cookie = ler_cookie(sessao).map_err(E::Cookie)?;
    let (html, mime) =
        obter_tela(&mut rede, &cookie, link.as_str()).map_err(|erro| match erro {
            FalhaDaTela::Transporte(falha) => E::Tela(falha),
            FalhaDaTela::Redirecionada => E::TelaDaAtribuicaoRedirecionada,
            FalhaDaTela::SessaoEncerrada => E::SessaoEncerradaNaAtribuicao,
        })?;
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(E::OrigemNaoAutorizada)?;
    let (valor, submissao) = preparar_submissao(
        &instalacao,
        link.as_str(),
        &html,
        &mime,
        &aberto.arvore.processo.id,
        unidade,
        pedido.usuario,
        Some(atual),
    )
    .map_err(E::Preparacao)?;
    let mut feito = ProcessoAtribuido {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: aberto.arvore.processo.id,
        unidade: unidade.to_owned(),
        usuario_atual: atual.to_owned(),
        usuario_pedido: pedido.usuario.map(str::to_owned),
        valor_da_opcao: valor,
        estado: EstadoDaAtribuicao::Previa,
        requisicoes: rede.feitas(),
    };
    if !pedido.confirmar {
        return Ok(feito);
    }

    // O envio consome o formulário uma única vez. Mesmo falhando a
    // resposta, a escrita pode ter ocorrido: reler, nunca repetir.
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, cookie| {
        enviado = true;
        rede.post(s, cookie)
    });
    // A releitura mede estado, mas não a autoria da mudança. Mesmo
    // quando o POST cai sem resposta, outro agente pode produzir o mesmo
    // resultado: só um 302 positivamente vinculado a esta escrita permite
    // confirmar depois da releitura. O resto é efeito incerto (saída 3).
    let (falha_do_envio, resposta_permite_confirmar) = match resposta {
        // A observação do SEI/UnB (Todos @323eb01) mostra 302 com
        // identidade explícita como aceite. HTML 200 genérico não vincula
        // este POST à alteração relida: outro agente pode tê-la feito.
        // Sem marcador positivo observado, falhar fechado (saída 3).
        Ok(RespostaDoEnvio::Html(_)) => (None, false),
        Ok(RespostaDoEnvio::Redirecionamento(redir)) => (
            None,
            redirecionamento_esperado(&redir, &feito.id_procedimento),
        ),
        // Uma falha de transporte após enviar o POST não comprova que
        // ele foi aplicado; guardar a causa e reler somente para
        // registrar o estado possivelmente residual, nunca para afirmar
        // autoria por coincidência.
        Err(erro) if enviado => (Some(erro), false),
        Err(erro) => return Err(E::Formulario(erro)),
    };

    // A leitura de prova faz GET novo, nunca reutiliza o estado anterior.
    let verificado = obter_processo(
        sessao,
        &leitura(pedido.nup, pedido.unidade, false),
        indice,
        &mut rede,
    );
    feito.requisicoes = rede.feitas();
    let (atribuicao_relida, leitura) = match verificado {
        Ok(lido) if lido.arvore.processo.id == feito.id_procedimento => (
            atribuicao_observada(&lido.arvore.aberto_em, &feito.unidade).map(str::to_owned),
            None,
        ),
        Ok(_) => (None, None),
        Err(erro) => (None, Some(erro)),
    };
    // A unidade precisa estar publicada: ausência da unidade NÃO significa
    // remoção da atribuição. Divergência após POST é efeito incerto (3).
    if !resposta_permite_confirmar
        || atribuicao_relida.as_deref() != Some(pedido.usuario.unwrap_or(""))
    {
        return Err(E::EfeitoIncertoNaAtribuicao {
            falha_do_envio: falha_do_envio.map(Box::new),
            leitura: leitura.map(Box::new),
            atribuicao_relida,
            requisicoes: feito.requisicoes,
        });
    }
    feito.estado = EstadoDaAtribuicao::AlteracaoConfirmada;
    Ok(feito)
}

/// Falhar fechado: a unidade da tela não pode ser substituída por uma
/// unidade da árvore. O diagnóstico só informa os fatos que foram lidos.
fn diagnostico_unidade_nao_observada(
    aberto_em: &AbertoEm,
    unidade: &str,
    requisicoes: u8,
) -> ErroProcessoAtribuir {
    let (siglas, motivo) = match aberto_em {
        AbertoEm::Unidades(unidades) => (
            Some(unidades.iter().map(|u| u.sigla.clone()).collect()),
            None,
        ),
        AbertoEm::Sobrestado(_) => (None, Some("raiz_publica_o_sobrestamento")),
        AbertoEm::NaoLido(motivo) => (None, Some(motivo.como_texto())),
    };
    ErroProcessoAtribuir::SituacaoDaAtribuicaoNaoObservada {
        unidade: unidade.to_owned(),
        aberto_em: siglas,
        motivo,
        requisicoes,
    }
}

/// None significa que a árvore não publicou a situação da unidade.
fn atribuicao_observada<'a>(aberto_em: &'a AbertoEm, unidade: &str) -> Option<&'a str> {
    match aberto_em {
        AbertoEm::Unidades(unidades) => unidades
            .iter()
            .find(|u| u.sigla == unidade)
            .map(|u| u.atribuido_a.as_ref().map_or("", |a| a.sigla.as_str())),
        AbertoEm::Sobrestado(_) | AbertoEm::NaoLido(_) => None,
    }
}

/// O 302 é apenas um dado, nunca seguido. Conferimos a ação do destino
/// e, quando publicada, a identidade do mesmo processo antes da releitura.
fn redirecionamento_esperado(resposta: &Redirecionamento302, id: &str) -> bool {
    let Some(consulta) = resposta
        .localizacao()
        .and_then(|url| consulta_do_controlador(url, resposta.url_do_envio()))
    else {
        return false;
    };
    let (mut acao, mut procedimento, mut origem) = (None, None, None);
    for item in consulta.split('&') {
        let Some((nome, valor)) = item.split_once('=') else {
            return false;
        };
        let destino = match nome {
            "acao" => &mut acao,
            "id_procedimento" => &mut procedimento,
            "acao_origem" => &mut origem,
            _ => continue,
        };
        if destino.replace(valor).is_some() {
            return false;
        }
    }
    // Tanto `arvore_visualizar` quanto `principal` são destinos
    // genéricos: sem ID explícito, uma leitura coincidente pode ser
    // atribuição/remoção concorrente de outro agente, não deste POST.
    // O 302 é apenas inspecionado, nunca seguido.
    let acao_esperada = matches!(acao, Some("arvore_visualizar" | "principal"));
    // O aceite observado publica também a ação que originou o 302.
    // Sem essa ligação ao POST de atribuição, a árvore coincidente
    // pode ter sido alterada por outro agente: não declarar sucesso.
    acao_esperada && procedimento == Some(id) && origem == Some("procedimento_atribuicao_cadastrar")
}
