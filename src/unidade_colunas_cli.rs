//! `ies unidade colunas`: as 12 opções do catálogo do Nível de detalhe, quais
//! a tela publica e quais estão marcadas, só leitura
//! (contracts/cli/unidade-colunas.md). Nada é salvo.
use crate::colheita_do_controle::{Colheita, EnvioColhendo};
use crate::indice_local::IndiceLocal;
use crate::transporte::envio_post::ler_tela_do_controle;
use crate::unidade_lista_cli::erro_da_visao;
use crate::visao_do_controle::{self, TelaAtual};
use crate::{configuracao, erro_fonte};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn mostrar_colunas(unidade: Option<&str>, json_mode: bool) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let colheita = Colheita::da_sessao(&indice, &arquivos);
    let tela =
        match ler_tela_do_controle(&arquivos.url_controle, &arquivos.cookie, &arquivos.origem) {
            Ok(tela) => tela,
            Err(falha) => {
                return crate::saida_erro::erro_fonte_http(
                    json_mode,
                    falha.subcodigo(),
                    falha.mensagem(),
                    falha.status_http(),
                );
            }
        };
    colheita.colher(&tela.html, Some(&tela.mime));
    let atual = TelaAtual {
        url: tela.url,
        html: tela.html,
        mime: tela.mime,
    };
    let (rotulo, nivel) = match visao_do_controle::ler_colunas(
        &tela.instalacao,
        &atual,
        unidade,
        &arquivos.cookie,
        &mut EnvioColhendo {
            colheita: &colheita,
        },
    ) {
        Ok(lido) => lido,
        Err(erro) => return erro_da_visao(json_mode, &erro),
    };
    if json_mode {
        let valor = json!({"unidade": rotulo, "origem": "web", "opcoes": nivel.json()});
        println!("{}", serde_json::to_string_pretty(&valor).unwrap());
    } else {
        println!("IES: Nível de detalhe da unidade {rotulo}:");
        for opcao in &nivel.opcoes {
            let (marca, nota) = match (opcao.disponivel, opcao.marcada) {
                (false, _) => ("[-]", " indisponível nesta instalação"),
                (true, true) => ("[x]", ""),
                (true, false) => ("[ ]", ""),
            };
            println!(
                "{marca} {} ({}){nota}",
                opcao.coluna.rotulo(),
                opcao.coluna.chave()
            );
        }
    }
    ExitCode::SUCCESS
}
