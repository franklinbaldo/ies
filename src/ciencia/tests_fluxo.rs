//! Orçamento exato de requisições de `processo ciente` com transporte
//! sintético; cenários do Todos 323eb01 (ciência do processo).
use super::{ErroCienciaProcesso as E, PedidoDaCiencia, ciencia_no_processo};
use super::FalhaDoAceiteProcesso;
use crate::processo_concluir::tests_apoio::*;
use crate::transporte::FalhaTransporte;

fn botao() -> Vec<String> {
    vec![ACAO_FUNCAO.replace("concluirProcesso();", "cienciaProcesso();")]
}

fn tela() -> String {
    tela_do_no(ID)
        .replace("concluirProcesso", "cienciaProcesso")
        .replace("procedimento_concluir", "procedimento_ciencia")
}

fn link_de_ciencia() -> String {
    absoluta(&link_da_funcao(ID).replace("procedimento_concluir", "procedimento_ciencia"))
}

fn link_da_lista() -> String {
    absoluta(&link_da_funcao(ID).replace("procedimento_concluir", "protocolo_ciencia_listar"))
}

fn arvore_com_ciencias() -> String {
    let base = arvore(&[UNIDADE], &botao());
    let link_relativo = link_da_lista().replace(&format!("{ORIGEM}/"), "");
    let acao = format!(
        "NosAcoes[0] = new infraArvoreAcao(\"CIENCIAS\",\"C\",\"{ID}\",\"{link_relativo}\");"
    );
    base.replacen("</script>", &format!("{acao}\n</script>"), 1)
}

fn aceite(atividade: &str) -> String {
    format!(
        "<html><head><script>atualizarArvore('controlador.php?acao=procedimento_visualizar&id_procedimento={ID}&procedimento_visualizar_ciencias=1&id_atividade={atividade}#ID-{atividade}');</script></head></html>"
    )
}

fn lista(atividade: &str) -> String {
    format!(
        "<html><table id='tblAndamentos'><caption>Lista de Ciências:</caption><tr><td><input type='checkbox' name='chkInfraItem1' value='{atividade}' /></td></tr></table></html>"
    )
}

fn rodar(
    confirmar: bool,
    respostas: Vec<Resposta>,
) -> (Result<super::CienciaDoProcesso, E>, Vec<Pedido>) {
    let amb = ambiente();
    rodar_com(&amb, respostas, |sessao, nup, indice, roteiro| {
        ciencia_no_processo(
            sessao,
            &PedidoDaCiencia {
                nup,
                unidade: None,
                confirmar,
            },
            indice,
            roteiro,
        )
    })
}

#[test]
fn previa_exatamente_tres_get_sem_get_mutador() {
    let (feito, pedidos) = rodar(false, abertura(arvore(&[UNIDADE], &botao())));
    let feito = feito.unwrap();
    assert!(!feito.confirmado);
    assert!(!feito.verificado);
    assert_eq!(feito.requisicoes, 3);
    assert_eq!(pedidos, pedidos_da_abertura());
}

#[test]
fn sem_botao_recusa_antes_de_escrever() {
    let (resultado, pedidos) = rodar(true, abertura(arvore(&[UNIDADE], vazio())));
    assert!(matches!(resultado, Err(E::CienciaNaoOferecida)));
    assert_eq!(pedidos, pedidos_da_abertura());
}

#[test]
fn confirmar_um_get_mutador_mais_arvore_e_lista_que_provam_a_atividade() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([
        ok(tela()),
        ok(aceite("42457")),
        ok(arvore_com_ciencias()),
        ok(lista("42457")),
    ]);
    let (feito, pedidos) = rodar(true, respostas);
    let feito = feito.unwrap();
    assert!(feito.verificado);
    assert_eq!(feito.id_atividade.as_deref(), Some("42457"));
    assert_eq!(feito.requisicoes, 7);
    let mut esperados = pedidos_da_abertura();
    esperados.extend([
        get(&links().1),
        get(&link_de_ciencia()),
        get(&url_arvore()),
        get(&link_da_lista()),
    ]);
    assert_eq!(pedidos, esperados);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
}

#[test]
fn falha_no_get_de_escrita_nao_dispara_retry_e_releitura_ocorre() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([
        ok(tela()),
        Err(FalhaTransporte::Rede),
        ok(arvore_com_ciencias()),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    let Err(E::EfeitoIncerto { requisicoes, .. }) = resultado else {
        panic!("escrita sem aceite exige saída 3");
    };
    assert_eq!(requisicoes, 6);
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

#[test]
fn atividade_nao_encontrada_na_lista_e_efeito_incerto_nao_sucesso() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([
        ok(tela()),
        ok(aceite("42457")),
        ok(arvore_com_ciencias()),
        ok(lista("42458")),
    ]);
    let (resultado, pedidos) = rodar(true, respostas);
    assert!(matches!(resultado, Err(E::EfeitoIncerto { .. })));
    assert_eq!(pedidos.len(), 7);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1
    );
}

#[test]
fn funcao_de_ciencia_ausente_na_tela_nao_escreve() {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.push(ok(tela().replace("cienciaProcesso", "outraFuncao")));
    let (resultado, pedidos) = rodar(true, respostas);
    assert!(matches!(resultado, Err(E::Barra(_))));
    assert_eq!(pedidos.len(), 4);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        0
    );
}

/// RED #271: o erro tipado do único GET de escrita do processo não pode ser
/// descartado enquanto a releitura independente tenta provar a atividade.
enum FalhaEsperada {
    Http403,
    Infra200,
    Rede,
}

fn conferir_falha_do_aceite_sem_retry(resposta: Resposta, causa: FalhaEsperada) {
    let mut respostas = abertura(arvore(&[UNIDADE], &botao()));
    respostas.extend([ok(tela()), resposta, ok(arvore_com_ciencias())]);
    let (resultado, pedidos) = rodar(true, respostas);
    let erro = resultado.unwrap_err();
    assert_eq!(erro.saida(), 3);
    match causa {
        FalhaEsperada::Http403 => {
            assert_eq!(erro.status_http().map(|s| s.status), Some(403));
            assert!(matches!(
                erro,
                E::EfeitoIncerto {
                    falha_do_aceite: Some(FalhaDoAceiteProcesso::Transporte(
                        FalhaTransporte::AcessoNegado(_)
                    )),
                    ..
                }
            ));
        }
        FalhaEsperada::Infra200 => {
            assert_eq!(erro.status_http().map(|s| s.status), Some(200));
            assert_eq!(
                erro.status_http()
                    .and_then(|s| s.mensagem_do_servidor.as_deref()),
                Some("Solicitacao recusada")
            );
            assert!(matches!(
                erro,
                E::EfeitoIncerto {
                    falha_do_aceite: Some(FalhaDoAceiteProcesso::ValidacaoDaInfra(_)),
                    ..
                }
            ));
        }
        FalhaEsperada::Rede => {
            assert!(erro.status_http().is_none());
            assert!(matches!(
                erro,
                E::EfeitoIncerto {
                    falha_do_aceite: Some(FalhaDoAceiteProcesso::Transporte(FalhaTransporte::Rede)),
                    ..
                }
            ));
        }
    }
    assert_eq!(pedidos.len(), 6);
    assert_eq!(
        pedidos
            .iter()
            .filter(|p| p.url == link_de_ciencia())
            .count(),
        1,
        "o GET potencialmente mutador não pode ser repetido"
    );
    assert_eq!(pedidos.last(), Some(&get(&url_arvore())));
}

#[test]
fn get_de_ciencia_http_403_preserva_a_recusa_tipificada() {
    conferir_falha_do_aceite_sem_retry(
        Ok((
            403,
            "<html><body>Solicitação recusada</body></html>".to_owned(),
            None,
        )),
        FalhaEsperada::Http403,
    );
}

#[test]
fn get_de_ciencia_infra_200_preserva_a_recusa_tipificada() {
    conferir_falha_do_aceite_sem_retry(
        ok(
            "<html><textarea id='txaInfraValidacao'>Solicitacao recusada</textarea></html>"
                .to_owned(),
        ),
        FalhaEsperada::Infra200,
    );
}

#[test]
fn get_de_ciencia_falha_de_rede_preserva_o_erro_tipificado() {
    conferir_falha_do_aceite_sem_retry(Err(FalhaTransporte::Rede), FalhaEsperada::Rede);
}
