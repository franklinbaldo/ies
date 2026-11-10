//! `ies unidade colunas`: as 12 opções do catálogo do Nível de detalhe, quais
//! a tela publica e quais estão marcadas, só leitura
//! (contracts/cli/unidade-colunas.md). Nada é salvo.
use crate::colheita_do_controle::{Colheita, EnvioColhendo};
use crate::indice_local::IndiceLocal;
use crate::transporte::envio_post::ler_tela_do_controle;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::{self, ErroVisaoDoControle, TelaAtual};
use crate::{configuracao, erro_fonte, relogin_cli};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn mostrar_colunas(unidade: Option<&str>, json_mode: bool) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.mensagem()),
    };
    // Cada tentativa relê a sessão num retrato coerente (#222) e colhe na
    // geração dele.
    let lido = relogin_cli::ler(&arquivos, unidade, || {
        let indice = IndiceLocal::da_configuracao(&arquivos);
        let sessao =
            ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)
                .map_err(ErroVisaoDoControle::Transporte)?;
        let colheita = Colheita::das_referencias(&indice, &sessao);
        let tela = ler_tela_do_controle(&sessao).map_err(ErroVisaoDoControle::Transporte)?;
        colheita.colher(&tela.html, Some(&tela.mime));
        let atual = TelaAtual {
            url: tela.url,
            html: tela.html,
            mime: tela.mime,
        };
        visao_do_controle::ler_colunas(
            &tela.instalacao,
            &atual,
            unidade,
            sessao.cookie(),
            &mut visao_do_controle::EnvioContado::novo(EnvioColhendo {
                colheita: &colheita,
            }),
        )
    });
    let ((rotulo, nivel), nota) = match lido {
        Ok(lido) => lido,
        Err(falha) => return relogin_cli::publicar_falha(json_mode, &falha),
    };
    if json_mode {
        let mut valor = json!({"unidade": rotulo, "origem": "web", "opcoes": nivel.json()});
        relogin_cli::anotar_com_total(&mut valor, nota.as_ref());
        if let Err(erro) = crate::saida::imprimir_json(&valor) {
            eprintln!("IES: falha ao emitir JSON: {erro}");
            return std::process::ExitCode::from(1);
        }
    } else {
        relogin_cli::avisar(json_mode, nota.as_ref());
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
