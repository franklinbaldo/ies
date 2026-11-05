//! #222: a leitura compartilhada da sessão é um retrato coerente. O
//! marcador `.geracao` é lido antes e depois da URL e do cookie; se mudou
//! (ou não confere com eles), os arquivos locais são relidos uma vez; se
//! mudou de novo, `sessao_trocada_durante_a_leitura`. Os escritores aqui
//! são os de produção: o login (#200) e a troca de unidade (#205),
//! intercalados de forma determinística entre a URL e o cookie.
use super::gravacao;
use super::tests_apoio::*;
use super::tests_colheita::controle_com_linha;
use crate::colheita_do_controle::Colheita;
use crate::indice_local::{ChaveDoLink, IndiceLocal};
use crate::instalacao::Instalacao;
use crate::transporte::FalhaTransporte;
use crate::transporte::sessao::ReferenciasDaSessao;

const COOKIE_A: &str = "PHPSESSID=GERACAO_A";
const COOKIE_B: &str = "PHPSESSID=GERACAO_B";
const COOKIE_C: &str = "PHPSESSID=GERACAO_C";

/// A URL do Controle de outra unidade (a troca de unidade, #205).
fn url_da_unidade(id: &str) -> String {
    url_do_controle().replace(
        "infra_unidade_atual=110000849",
        &format!("infra_unidade_atual={id}"),
    )
}

fn ler(ambiente: &Ambiente, entre: impl FnMut()) -> Result<ReferenciasDaSessao, FalhaTransporte> {
    let arquivos = &ambiente.arquivos;
    ReferenciasDaSessao::ler_intercalado(
        &arquivos.url_controle,
        &arquivos.origem,
        &arquivos.cookie,
        entre,
    )
}

/// O par lido é de uma geração só: o marcador gravado confere com ele.
fn mesma_geracao(ambiente: &Ambiente, lidas: &ReferenciasDaSessao) -> bool {
    gravacao::mesma_geracao(
        &ambiente.arquivos,
        lidas.url_do_controle(),
        lidas.cookie().valor().unwrap(),
    )
}

fn logar(ambiente: &Ambiente, url: &str, cookie: &str) {
    gravacao::gravar(&ambiente.arquivos, ORIGEM, url, cookie).unwrap();
}

#[test]
fn relogin_entre_url_e_cookie_rele_e_entrega_a_geracao_nova_inteira() {
    // Antes da #222 (reprodutor da #224): URL A com cookie B.
    let ambiente = Ambiente::novo();
    logar(&ambiente, &url_do_controle(), COOKIE_A);
    let nova = url_da_unidade("110000850");
    let mut vezes = 0;
    let lidas = ler(&ambiente, || {
        vezes += 1;
        if vezes == 1 {
            logar(&ambiente, &nova, COOKIE_B);
        }
    })
    .unwrap();
    assert_eq!(vezes, 2, "uma releitura local, nenhuma a mais");
    assert_eq!(lidas.url_do_controle(), nova);
    assert_eq!(lidas.cookie().valor(), Ok(COOKIE_B));
    assert!(mesma_geracao(&ambiente, &lidas));
}

#[test]
fn troca_de_unidade_entre_url_e_cookie_rele_e_entrega_a_url_nova() {
    // Antes da #222 (reprodutor da #224): URL antiga com o marcador novo.
    let ambiente = Ambiente::novo();
    logar(&ambiente, &url_do_controle(), COOKIE_A);
    let nova = url_da_unidade("110000850");
    let mut vezes = 0;
    let lidas = ler(&ambiente, || {
        vezes += 1;
        if vezes == 1 {
            gravacao::regravar_url_do_controle(&ambiente.arquivos, &nova, COOKIE_A).unwrap();
        }
    })
    .unwrap();
    assert_eq!(vezes, 2);
    assert_eq!(lidas.url_do_controle(), nova);
    assert_eq!(lidas.cookie().valor(), Ok(COOKIE_A));
    assert!(mesma_geracao(&ambiente, &lidas));
}

#[test]
fn geracao_trocada_nas_duas_leituras_falha_fechado_com_subcodigo_proprio() {
    let ambiente = Ambiente::novo();
    logar(&ambiente, &url_do_controle(), COOKIE_A);
    let mut vezes = 0;
    let falha = ler(&ambiente, || {
        vezes += 1;
        let (url, cookie) = if vezes == 1 {
            (url_da_unidade("110000850"), COOKIE_B)
        } else {
            (url_da_unidade("110000851"), COOKIE_C)
        };
        logar(&ambiente, &url, cookie);
    })
    .unwrap_err();
    assert_eq!(vezes, 2, "uma releitura só, depois falha");
    assert_eq!(falha, FalhaTransporte::SessaoTrocadaDuranteALeitura);
    assert_eq!(falha.subcodigo(), "sessao_trocada_durante_a_leitura");
    assert!(!crate::sessao_login::relogin::sessao_encerrada(
        falha.subcodigo()
    ));
}

#[test]
fn par_que_nao_confere_com_o_marcador_estavel_nunca_e_entregue() {
    // Gravação interrompida entre os `rename`: cookie novo, marcador velho.
    let ambiente = Ambiente::novo();
    logar(&ambiente, &url_do_controle(), COOKIE_A);
    privado(&ambiente.arquivos.cookie, &format!("{COOKIE_B}\n"));
    let mut vezes = 0;
    let falha = ler(&ambiente, || vezes += 1).unwrap_err();
    assert_eq!(vezes, 2);
    assert_eq!(falha, FalhaTransporte::SessaoTrocadaDuranteALeitura);
}

#[test]
fn sessao_sem_marcador_mantem_a_leitura_de_hoje_lacuna_documentada() {
    // Sessão preparada fora do `ies login`: não há geração para comparar.
    let ambiente = Ambiente::novo();
    ambiente.gravar_sessao(&url_do_controle(), COOKIE_A);
    let mut vezes = 0;
    let lidas = ler(&ambiente, || {
        vezes += 1;
        privado(&ambiente.arquivos.cookie, &format!("{COOKIE_B}\n"));
    })
    .unwrap();
    assert_eq!(vezes, 1, "sem marcador, nenhuma releitura");
    assert_eq!(lidas.url_do_controle(), url_do_controle());
    assert_eq!(lidas.cookie().valor(), Ok(COOKIE_B));
}

#[test]
fn origem_divergente_e_recusada_antes_de_abrir_o_cookie() {
    let ambiente = Ambiente::novo();
    logar(&ambiente, &url_do_controle(), COOKIE_A);
    privado(
        &ambiente.arquivos.origem,
        "https://outro.exemplo.gov.br/sei\n",
    );
    std::fs::remove_file(&ambiente.arquivos.cookie).unwrap();
    // A recusa fica no lugar do cookie: aparece na primeira requisição,
    // e o arquivo removido não foi aberto (seria `ausente`).
    let lidas = ler(&ambiente, || {}).unwrap();
    assert_eq!(
        lidas.cookie().valor(),
        Err(FalhaTransporte::OrigemNaoAutorizada)
    );
}

#[test]
fn colheita_da_leitura_registra_sob_a_geracao_do_cookie_lido() {
    // O Controle foi obtido com o par A; um relogin grava B antes da
    // colheita. O link vai para a geração A, nunca para a B.
    let ambiente = Ambiente::novo();
    logar(&ambiente, &url_do_controle(), COOKIE_A);
    let lidas = ler(&ambiente, || {}).unwrap();
    logar(&ambiente, &url_do_controle(), COOKIE_B);
    let indice = IndiceLocal::em(ambiente.dir.join("indice").join("ies.sqlite3"));
    Colheita::das_referencias(&indice, &lidas).colher(&controle_com_linha(), Some("text/html"));
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    let chave = |geracao| ChaveDoLink {
        geracao_sessao: geracao,
        unidade: "110000849",
        acao: "procedimento_trabalhar",
        objeto: "77",
    };
    let a = super::geracao_da_sessao(COOKIE_A);
    let b = super::geracao_da_sessao(COOKIE_B);
    assert!(indice.link(&instalacao, chave(&a)).is_some());
    assert!(indice.link(&instalacao, chave(&b)).is_none());
}
