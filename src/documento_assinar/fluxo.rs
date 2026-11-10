//! A sequência de `ies documento assinar` (contracts/cli/documento-assinar.md):
//! abertura até a árvore, decisão antes de escrever, um POST e a leitura de
//! verificação, com a tabela de reconciliação do contrato. Nenhum 302 é
//! seguido; nenhuma escrita é repetida; a senha só entra no corpo do POST.
use super::erro::{ErroAssinar as A, FalhaDoEnvio, VerificacaoDaAssinatura as Vf};
use super::tela::{
    ASSINAR_DOCUMENTO, CARGO, FORMA_DE_AUTENTICACAO, ORGAO, PELA_SENHA, SENHA, TelaDeAssinatura,
    USUARIO, assinatura_realizada, conferir_tela_de_assinatura,
};
use super::{Confirmacao, DocumentoAssinado, PedidoDaAssinatura, ResultadoDaAssinatura};
use crate::arvore_processo::{
    Assinante, EdicaoNaBarra, ler_assinatura_na_arvore, ler_barra_do_documento,
    ler_link_de_assinar, tela_do_documento,
};
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    ErroSubmissaoFormulario, RespostaDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao,
    enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, ler_cookie, obter_tela};
use crate::processo_obter::{
    DocumentoNaArvoreAberto, EnvioDoProcesso, ErroDocumentoObter, ErroProcessoObter, PedidoDoAnexo,
    PedidoDoDocumento, SessaoDoProcesso, abrir_documento_na_arvore,
};
use crate::transporte::FalhaTransporte;
use crate::visao_do_controle::EnvioDoControle as _;

/// Prévia: abertura + 2 GET, sem POST. Escrita: um POST, nunca repetido.
pub(crate) fn assinar_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaAssinatura<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<DocumentoAssinado, A> {
    let instalacao =
        Instalacao::interpretar(sessao.origem).ok_or(A::Abertura(ErroDocumentoObter::Processo(
            ErroProcessoObter::Transporte(FalhaTransporte::OrigemNaoAutorizada),
        )))?;
    let mut rede = Contador::novo(envio);
    let leitura = PedidoDoDocumento {
        numero: pedido.numero,
        unidade: pedido.unidade,
        processo: pedido.processo,
        anexo: PedidoDoAnexo::default(),
    };
    let aberto = abrir_documento_na_arvore(sessao, &leitura, indice, &mut rede)?;
    let (id, id_processo) = (aberto.id_documento.clone(), aberto.processo.id.clone());
    let na_arvore = ler_assinatura_na_arvore(&aberto.html, &id)
        .map_err(crate::documento_assinar::erro::ErroAssinar::Arvore)?;
    if !na_arvore.publicada {
        return Err(A::AssinaturaNaoPublicadaNaBarra);
    }
    let tela_do_doc =
        tela_do_documento(&instalacao, &aberto.html, &id, &id_processo).ok_or(A::Html(
            ErroHtml::valor("endereco_da_tela_do_documento_nao_reconhecido").com_mensagem(
                "a árvore não publica a tela do documento na forma observada; nada foi assinado",
            ),
        ))?;
    let cookie = ler_cookie(sessao)
        .map_err(|falha| A::Formulario(ErroSubmissaoFormulario::Transporte(falha)))?;
    let (html, _) = obter_tela(&mut rede, &cookie, tela_do_doc.as_str()).map_err(da_tela)?;
    let link = ler_link_de_assinar(&instalacao, &html, &id, &id_processo)?;
    let (html_da_tela, mime) = obter_tela(&mut rede, &cookie, link.as_str()).map_err(da_tela)?;
    let tela = conferir_tela_de_assinatura(&html_da_tela, &id)?;
    let mut feito = previa(&instalacao, &aberto, &tela, na_arvore.assinantes, pedido);
    feito.requisicoes = rede.feitas();
    if feito.assinaturas.iter().any(|a| a.nome == tela.assinante) {
        feito.resultado = ResultadoDaAssinatura::JaAssinadoPelaSessao {
            confirmacao_pedida: pedido.confirmacao.is_some(),
        };
        return Ok(feito);
    }
    if let Some(cargo) = &feito.cargo
        && !tela.cargos.contains(cargo)
    {
        return Err(A::CargoNaoOferecidoPelaTela {
            cargos: tela.cargos,
        });
    }
    if !feito.assinaturas.is_empty() {
        return Err(A::DocumentoAssinadoPorOutroUsuario {
            antes: feito.assinaturas,
        });
    }
    let Some(confirmacao) = pedido.confirmacao else {
        return Ok(feito);
    };
    let codificacao = codificacao_da_tela(&mime).ok_or(A::CharsetDaTelaNaoSuportado)?;
    let servida = TelaServida {
        url: link.as_str(),
        html: &html_da_tela,
        codificacao,
    };
    let submissao = derivar(&instalacao, &servida, &tela, &feito, confirmacao)?;
    let falha = enviar(&mut rede, sessao, submissao)?;
    let (verificacao, edicao_depois) = verificar(&instalacao, sessao, &leitura, indice, &mut rede);
    feito.requisicoes = rede.feitas();
    conciliar(feito, falha, verificacao, edicao_depois)
}

/// O sucesso da prévia a partir do que as três leituras mostraram.
fn previa(
    instalacao: &Instalacao,
    aberto: &DocumentoNaArvoreAberto,
    tela: &TelaDeAssinatura,
    assinaturas: Vec<Assinante>,
    pedido: &PedidoDaAssinatura<'_>,
) -> DocumentoAssinado {
    let cargo = pedido
        .confirmacao
        .map(|c| c.cargo)
        .or(pedido.cargo)
        .map(str::to_owned);
    DocumentoAssinado {
        numero: aberto.numero.clone(),
        id_documento: aberto.id_documento.clone(),
        rotulo: aberto.rotulo.clone(),
        processo: aberto.processo.clone(),
        unidade: aberto.unidade.clone(),
        assinante: tela.assinante.clone(),
        cargo,
        cargo_da_tela: tela.cargo_da_tela.clone(),
        cargos: tela.cargos.clone(),
        assinaturas,
        edicao: edicao(instalacao, aberto),
        resultado: ResultadoDaAssinatura::Previa,
        requisicoes: 0,
    }
}

/// A edição da barra (`editarConteudo`), informativa: barra recusada pelo
/// leitor das versões é `fora_da_forma`, não erro.
fn edicao(instalacao: &Instalacao, aberto: &DocumentoNaArvoreAberto) -> EdicaoNaBarra {
    ler_barra_do_documento(
        instalacao,
        &aberto.html,
        &aberto.id_documento,
        &aberto.processo.id,
    )
    .map_or(EdicaoNaBarra::ForaDaForma, |barra| barra.edicao)
}

fn da_tela(falha: FalhaDaTela) -> A {
    match falha {
        FalhaDaTela::Transporte(falha) => A::TelaDaEscrita(falha),
        FalhaDaTela::Redirecionada => A::TelaDaEscritaRedirecionada,
        FalhaDaTela::SessaoEncerrada => A::SessaoEncerrada,
    }
}

/// O POST pelo primitivo, sem rede: os campos servidos, o cargo e a senha.
fn derivar(
    instalacao: &Instalacao,
    servida: &TelaServida<'_>,
    tela: &TelaDeAssinatura,
    feito: &DocumentoAssinado,
    confirmacao: Confirmacao<'_>,
) -> Result<SubmissaoDerivada, A> {
    let campos = [
        (FORMA_DE_AUTENTICACAO, PELA_SENHA),
        (ORGAO, tela.orgao.as_str()),
        (USUARIO, tela.assinante.as_str()),
        (CARGO, confirmacao.cargo),
        (SENHA, confirmacao.credenciais.senha()),
    ];
    let unidade = feito.unidade.as_deref().unwrap_or_default();
    derivar_submissao(&ASSINAR_DOCUMENTO, instalacao, servida, unidade, &campos)
        .map_err(A::Formulario)
}

/// Um único POST. `Ok(None)`: a tela com `finalizar()`; `Ok(Some(falha))`:
/// o pedido saiu e a resposta não é a esperada; `Err`: nada saiu.
fn enviar<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    submissao: SubmissaoDerivada,
) -> Result<Option<FalhaDoEnvio>, A> {
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |s, c| {
        enviado = true;
        rede.post(s, c)
    });
    match resposta {
        Ok(RespostaDoEnvio::Html(html)) => {
            Ok((!assinatura_realizada(&html)).then_some(FalhaDoEnvio::RespostaSemFinalizacao))
        }
        Ok(RespostaDoEnvio::Redirecionamento(_)) => Ok(Some(FalhaDoEnvio::Redirecionamento)),
        Err(erro) if enviado => Ok(Some(FalhaDoEnvio::Envio(erro))),
        Err(erro) => Err(A::Formulario(erro)),
    }
}

/// A leitura nova da árvore (o caminho mais barato de `documento DOC`),
/// feita uma vez.
fn verificar<V: EnvioDoProcesso>(
    instalacao: &Instalacao,
    sessao: &SessaoDoProcesso<'_>,
    leitura: &PedidoDoDocumento<'_>,
    indice: &IndiceLocal,
    rede: &mut Contador<'_, V>,
) -> (Vf, Option<EdicaoNaBarra>) {
    match abrir_documento_na_arvore(sessao, leitura, indice, rede) {
        Ok(aberto) => {
            let edicao = Some(edicao(instalacao, &aberto));
            match ler_assinatura_na_arvore(&aberto.html, &aberto.id_documento) {
                Ok(lida) => (Vf::Lida(lida.assinantes), edicao),
                Err(erro) => (Vf::ArvoreNaoLida(erro), edicao),
            }
        }
        Err(erro) => (Vf::LeituraFalhou(erro), None),
    }
}

/// Tabela de reconciliação do contrato (antes: nenhuma assinatura).
fn conciliar(
    mut feito: DocumentoAssinado,
    falha: Option<FalhaDoEnvio>,
    verificacao: Vf,
    edicao_depois: Option<EdicaoNaBarra>,
) -> Result<DocumentoAssinado, A> {
    let antes = feito.assinaturas.clone();
    let Vf::Lida(depois) = &verificacao else {
        return Err(A::EfeitoIncertoNaAssinatura {
            antes,
            falha_do_envio: falha,
            verificacao: Box::new(verificacao),
        });
    };
    let Some(edicao_depois) = edicao_depois else {
        return Err(A::EfeitoIncertoNaAssinatura {
            antes,
            falha_do_envio: falha,
            verificacao: Box::new(verificacao),
        });
    };
    if *depois == antes {
        return Err(A::AssinaturaNaoAplicada {
            antes,
            falha_do_envio: falha,
        });
    }
    let nova = match depois.as_slice() {
        [nova] if nova.nome == feito.assinante && Some(&nova.cargo) == feito.cargo.as_ref() => {
            nova.clone()
        }
        _ => {
            return Err(A::EfeitoIncertoNaAssinatura {
                antes,
                falha_do_envio: falha,
                verificacao: Box::new(verificacao),
            });
        }
    };
    feito.resultado = ResultadoDaAssinatura::Comprovada {
        assinatura_depois: nova,
        edicao_depois,
        reconciliado_por: falha,
    };
    Ok(feito)
}
