//! Prévia Web injetável da atribuição de processo: abertura, unidade,
//! atribuição publicada na árvore, formulário da barra e payload local.
//! A CLI mantém --confirmar fechado até haver validação live em treinamento.
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{AbertoEm, ErroAtribuirNaBarra};
use crate::formulario_web::{
    ErroSubmissaoFormulario, Redirecionamento302, RespostaDoEnvio, consulta_do_controlador,
    enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_atribuir::{Atribuicao, ErroAtribuir, preparar_submissao};
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever, obter_processo,
};
use crate::transporte::FalhaTransporte;
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
    /// `None`: a unidade não tinha atribuição (fato da árvore).
    pub(crate) usuario_atual: Option<String>,
    pub(crate) usuario_pedido: Option<String>,
    pub(crate) valor_da_opcao: String,
    pub(crate) estado: EstadoDaAtribuicao,
    pub(crate) requisicoes: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroProcessoAtribuir {
    Abertura(#[from] ErroProcessoObter),
    UnidadeDaSessaoNaoPublicada,
    /// A árvore publicou outras unidades, mas não a da sessão autenticada.
    UnidadeNaoPublicadaNaArvore {
        unidade: String,
        unidades_da_arvore: Vec<String>,
        requisicoes: u8,
    },
    /// A árvore está sobrestada e não publica a situação da unidade.
    ArvoreSobrestadaSemSituacaoDaUnidade {
        unidade: String,
        requisicoes: u8,
    },
    /// A árvore não foi lida, e o motivo da leitura fica registrado.
    SituacaoDaArvoreNaoLida {
        unidade: String,
        motivo: &'static str,
        requisicoes: u8,
    },
    AtribuicaoNaoOferecidaNaBarra,
    Barra(#[from] ErroAtribuirNaBarra),
    Cookie(#[from] CookieIlegivel),
    Tela(#[from] FalhaTransporte),
    TelaDaAtribuicaoRedirecionada,
    SessaoEncerradaNaAtribuicao,
    OrigemNaoAutorizada,
    Preparacao(#[from] ErroAtribuir),
    Formulario(#[from] ErroSubmissaoFormulario),
    /// A releitura da árvore não leu a unidade da sessão.
    ReleituraNaoLida {
        requisicoes: u8,
    },
    /// A releitura da árvore mostrou outro processo.
    ReleituraDeOutroProcesso {
        requisicoes: u8,
    },
    NaoConfirmadoNaAtribuicao {
        falha_do_envio: Option<Box<ErroSubmissaoFormulario>>,
        leitura: Option<Box<ErroProcessoObter>>,
        /// `Ok(None)` = unidade sem atribuição observada; `Ok(Some)` = login; `Err` = motivo da releitura.
        atribuicao_relida: Result<Option<String>, Box<ErroProcessoAtribuir>>,
        requisicoes: u8,
    },
}

/// A primeira requisição só ocorre depois da conferência da trava de escrita.
pub(crate) fn executar_previa<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaAtribuicao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoAtribuido, ErroProcessoAtribuir> {
    let mut rede = Contador::novo(envio);
    let abertura = leitura(pedido.nup, pedido.unidade, true);
    let (aberto, acoes) = abrir_para_escrever(sessao, &abertura, indice, &mut rede)?;
    let unidade = aberto
        .unidade
        .as_deref()
        .ok_or(ErroProcessoAtribuir::UnidadeDaSessaoNaoPublicada)?;
    let atual = situacao_da_unidade(&aberto.arvore.aberto_em, unidade, rede.feitas())?;
    // A árvore já informa quando a operação não alteraria nada.
    // Rejeitar antes de abrir o formulário evita um GET desnecessário,
    // inclusive nas confirmações injetáveis, sem mudar o estado remoto.
    let ja_aplicada = match pedido.usuario {
        Some(login) => !login.is_empty() && atual == Atribuicao::Usuario(login),
        None => atual == Atribuicao::Ninguem,
    };
    if ja_aplicada {
        return Err(ErroProcessoAtribuir::Preparacao(
            ErroAtribuir::AtribuicaoJaAplicada,
        ));
    }
    let link = acoes
        .atribuir?
        .ok_or(ErroProcessoAtribuir::AtribuicaoNaoOferecidaNaBarra)?;
    let cookie = ler_cookie(sessao)?;
    let (html, mime) =
        obter_tela(&mut rede, &cookie, link.as_str()).map_err(|erro| match erro {
            FalhaDaTela::Transporte(falha) => ErroProcessoAtribuir::Tela(falha),
            FalhaDaTela::Redirecionada => ErroProcessoAtribuir::TelaDaAtribuicaoRedirecionada,
            FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
                ErroProcessoAtribuir::SessaoEncerradaNaAtribuicao
            }
        })?;
    let instalacao =
        Instalacao::interpretar(sessao.origem).ok_or(ErroProcessoAtribuir::OrigemNaoAutorizada)?;
    let (valor, submissao) = preparar_submissao(
        &instalacao,
        &crate::formulario_web::TelaServida {
            url: link.as_str(),
            html: &html,
            codificacao: crate::pagina_do_controle::codificacao_da_tela(&mime)
                .ok_or(ErroAtribuir::CharsetDaTelaNaoSuportado)?,
        },
        &aberto.arvore.processo.id,
        unidade,
        pedido.usuario,
        atual,
    )?;
    let mut feito = ProcessoAtribuido {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: aberto.arvore.processo.id,
        unidade: unidade.to_owned(),
        usuario_atual: atual.login().map(str::to_owned),
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
        Err(erro) => return Err(ErroProcessoAtribuir::Formulario(erro)),
    };

    // A leitura de prova faz GET novo, nunca reutiliza o estado anterior.
    let (atribuicao_relida, leitura) = reler_situacao(sessao, pedido, indice, &mut rede, &feito);
    feito.requisicoes = rede.feitas();
    // A unidade precisa estar publicada: ausência da unidade NÃO significa
    // remoção da atribuição. Divergência após POST é efeito incerto (3).
    let confirmada = atribuicao_relida
        .as_ref()
        .is_ok_and(|relida| relida.as_deref() == pedido.usuario);
    if !resposta_permite_confirmar || !confirmada {
        return Err(ErroProcessoAtribuir::NaoConfirmadoNaAtribuicao {
            falha_do_envio: falha_do_envio.map(Box::new),
            leitura: leitura.map(Box::new),
            atribuicao_relida,
            requisicoes: feito.requisicoes,
        });
    }
    feito.estado = EstadoDaAtribuicao::AlteracaoConfirmada;
    Ok(feito)
}

/// Relê o processo depois do POST e devolve a situação da unidade da sessão,
/// ou o motivo de não ter sido possível lê-la.
fn reler_situacao<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaAtribuicao<'_>,
    indice: &IndiceLocal,
    rede: &mut Contador<'_, V>,
    feito: &ProcessoAtribuido,
) -> (
    Result<Option<String>, Box<ErroProcessoAtribuir>>,
    Option<ErroProcessoObter>,
) {
    let verificado = obter_processo(
        sessao,
        &leitura(pedido.nup, pedido.unidade, false),
        indice,
        rede,
    );
    // A contagem inclui o GET de prova, feito mesmo quando a releitura falha.
    let requisicoes = rede.feitas();
    match verificado {
        Ok(lido) if lido.arvore.processo.id == feito.id_procedimento => (
            situacao_da_unidade(&lido.arvore.aberto_em, &feito.unidade, requisicoes)
                .map(|atual| atual.login().map(str::to_owned))
                .map_err(Box::new),
            None,
        ),
        Ok(_) => (
            Err(Box::new(ErroProcessoAtribuir::ReleituraDeOutroProcesso {
                requisicoes,
            })),
            None,
        ),
        Err(erro) => (
            Err(Box::new(ErroProcessoAtribuir::ReleituraNaoLida {
                requisicoes,
            })),
            Some(erro),
        ),
    }
}

/// Falhar fechado: a unidade da tela não pode ser substituída por uma
/// unidade da árvore. Cada causa de ausência tem o seu erro tipado.
fn situacao_da_unidade<'a>(
    aberto_em: &'a AbertoEm,
    unidade: &str,
    requisicoes: u8,
) -> Result<Atribuicao<'a>, ErroProcessoAtribuir> {
    match aberto_em {
        AbertoEm::Unidades(unidades) => {
            let achada = unidades
                .iter()
                .find(|u| u.sigla == unidade)
                .ok_or_else(|| ErroProcessoAtribuir::UnidadeNaoPublicadaNaArvore {
                    unidade: unidade.to_owned(),
                    unidades_da_arvore: unidades.iter().map(|u| u.sigla.clone()).collect(),
                    requisicoes,
                })?;
            // Unidade sem atribuição é um fato publicado pela árvore, não ausência.
            Ok(achada
                .atribuido_a
                .as_ref()
                .map_or(Atribuicao::Ninguem, |a| {
                    Atribuicao::Usuario(a.sigla.as_str())
                }))
        }
        AbertoEm::Sobrestado(_) => {
            Err(ErroProcessoAtribuir::ArvoreSobrestadaSemSituacaoDaUnidade {
                unidade: unidade.to_owned(),
                requisicoes,
            })
        }
        AbertoEm::NaoLido(motivo) => Err(ErroProcessoAtribuir::SituacaoDaArvoreNaoLida {
            unidade: unidade.to_owned(),
            motivo: motivo.como_texto(),
            requisicoes,
        }),
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
    let acao_esperada = matches!(
        acao.and_then(AcaoDoSei::ler),
        Some(AcaoDoSei::ArvoreVisualizar | AcaoDoSei::Principal)
    );
    // O aceite observado publica também a ação que originou o 302.
    // Sem essa ligação ao POST de atribuição, a árvore coincidente
    // pode ter sido alterada por outro agente: não declarar sucesso.
    acao_esperada
        && procedimento == Some(id)
        && origem.and_then(AcaoDoSei::ler) == Some(AcaoDoSei::ProcedimentoAtribuicaoCadastrar)
}
