//! BDD da prévia opt-in com leitura independente da anotação do processo.
//! URLs, ids e textos sintéticos; sem acesso ao SEI.
use super::tests_apoio::*;
use super::{PedidoDoEnvio, enviar_processo};
use crate::arvore_processo::tests_concluir::absoluta;
use crate::processo_concluir::tests_apoio::*;
use crate::propriedades_processo::tests_apoio::acao_da_barra;

fn link_anotacao() -> String {
    format!(
        "controlador.php?acao=anotacao_registrar&acao_origem=arvore_visualizar&id_procedimento={ID}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH_ENVIAR}"
    )
}

fn anotacao(texto: &str, id: &str) -> String {
    format!(
        r#"<html><form id="frmAnotacaoCadastro" method="post" action="controlador.php?acao=anotacao_registrar">
          <input name="hdnIdProtocolo" type="hidden" value="{id}">
          <textarea id="txaDescricao" name="txaDescricao">{texto}</textarea></form></html>"#
    )
}

fn previa_com_anotacao(
    texto: &str,
    id: &str,
) -> (
    Result<super::ProcessoEnviado, super::ErroEnviar>,
    Vec<Pedido>,
) {
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela_de_envio(Versao::V40, ID)),
            ok(anotacao(texto, id)),
            ok(um_destino()),
        ],
    );
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoDoEnvio {
            nup,
            unidade: None,
            para: DESTINO,
            manter_aberto: false,
            remover_anotacoes: true,
            confirmar: false,
        };
        enviar_processo(sessao, &pedido, indice, rede)
    })
}

#[test]
fn bdd_previa_remocao_observa_anotacao_sem_escrever() {
    // Dado link assinado de anotações na barra e texto servido no cadastro,
    // quando executo ies processo enviar --remover-anotacoes,
    // então a prévia registra a presença e só faz GET de leitura.
    let (resultado, pedidos) = previa_com_anotacao("Minha anotação", ID);
    let feito = resultado.expect("a anotação foi lida");
    assert_eq!(feito.anotacao_antes, Some(true));
    assert_eq!(feito.requisicoes, 6);
    let mut esperado = pedidos_da_abertura();
    esperado.push(get(&url_da_tela()));
    esperado.push(get(&absoluta(&link_anotacao())));
    esperado.push(post(
        &absoluta(&autocompletar()),
        &corpo_da_consulta(Versao::V40),
    ));
    assert_eq!(pedidos, esperado);
}

#[test]
fn bdd_anotacao_de_outro_processo_recusa_antes_de_consultar_destino() {
    // Dado formulario de anotações de outro processo,
    // quando solicito prévia com remoção, então nenhuma consulta à unidade
    // de destino nem escrita é executada.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![
            ok(tela_de_envio(Versao::V40, ID)),
            ok(anotacao("Outra anotação", "999")),
        ],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoDoEnvio {
            nup,
            unidade: None,
            para: DESTINO,
            manter_aberto: false,
            remover_anotacoes: true,
            confirmar: false,
        };
        enviar_processo(sessao, &pedido, indice, rede)
    });
    assert!(resultado.is_err());
    let mut esperado = pedidos_da_abertura();
    esperado.push(get(&url_da_tela()));
    esperado.push(get(&absoluta(&link_anotacao())));
    assert_eq!(pedidos, esperado);
}

#[test]
fn bdd_sem_checkbox_mas_com_anotacao_planeja_edicao_na_previa() {
    // Dado a tela de envio sem o checkbox e o formulario da anotacao servido,
    // quando executo ies processo enviar --remover-anotacoes sem --confirmar,
    // entao o plano escolhe limpar a anotacao e nao faz escrita.
    let acoes = vec![pelo_envio(ID), acao_da_barra(&link_anotacao())];
    let tela = tela_de_envio(Versao::V41, ID)
        .replace(r#"name="chkSinRemoverAnotacoes""#, r#"name="""#);
    let respostas = com(
        abertura(arvore(&[UNIDADE], &acoes)),
        vec![ok(tela), ok(anotacao("Anotação de teste", ID)), ok(um_destino())],
    );
    let amb = ambiente();
    let (resultado, pedidos) = rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoDoEnvio {
            nup,
            unidade: None,
            para: DESTINO,
            manter_aberto: false,
            remover_anotacoes: true,
            confirmar: false,
        };
        enviar_processo(sessao, &pedido, indice, rede)
    });
    let feito = resultado.expect("a edicao da anotacao foi oferecida");
    assert_eq!(feito.anotacao_antes, Some(true));
    assert_eq!(
        feito.metodo_remocao,
        Some(super::MetodoRemocao::LimparAnotacao)
    );
    assert_eq!(feito.requisicoes, 6);
    let mut esperado = pedidos_da_abertura();
    esperado.push(get(&url_da_tela()));
    esperado.push(get(&absoluta(&link_anotacao())));
    esperado.push(post(
        &absoluta(&autocompletar()),
        &corpo_da_consulta(Versao::V41),
    ));
    assert_eq!(pedidos, esperado);
}
