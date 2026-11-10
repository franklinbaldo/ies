//! Borda da CLI "ies unidade retornos" (leitura, nunca POST).
use crate::colheita_do_controle::Colheita;
use crate::indice_local::IndiceLocal;
use crate::transporte::{self, sessao::ReferenciasDaSessao};
use crate::unidade_retornos_web::{self, ErroLeituraRetornos};
use crate::{configuracao, controle_reconhecido, relogin_cli};
use serde_json::json;

/// `alvo`: o mês já conferido pelo clap (`--mes`).
pub(super) fn listar(
    unidade: Option<&str>,
    json_mode: bool,
    alvo: Option<(u16, u8)>,
) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    let ((atual, resultado), nota) = relogin_cli::ler(&arquivos, unidade, || {
        let sessao =
            ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)
                .map_err(ErroLeituraRetornos::Transporte)?;
        let controle = transporte::envio_post::ler_tela_do_controle(&sessao)
            .map_err(ErroLeituraRetornos::Transporte)?;
        let unidade_atual = controle_reconhecido::unidade_do_controle(&controle.html)
            .map_err(ErroLeituraRetornos::Controle)?;
        if unidade.is_some_and(|esperada| esperada != unidade_atual) {
            return Err(ErroLeituraRetornos::UnidadeDiferenteDaSolicitada);
        }
        let indice = IndiceLocal::da_configuracao(&arquivos);
        Colheita::das_referencias(&indice, &sessao).colher(&controle.html, Some(&controle.mime));
        let resultado = unidade_retornos_web::ler_com(
            &controle.url,
            &controle.html,
            &controle.instalacao,
            alvo,
            |url| {
                // A URL ja passou por conferir_link: controlador, mesma
                // instalacao, acao, hash e parametros sem duplicidade.
                let cookie = sessao
                    .cookie()
                    .valor()
                    .map_err(ErroLeituraRetornos::Transporte)?;
                let resposta =
                    transporte::get_https(url, cookie).map_err(ErroLeituraRetornos::Transporte)?;
                transporte::decodificar_resposta(resposta).map_err(ErroLeituraRetornos::Transporte)
            },
        )?;
        Ok((unidade_atual, resultado))
    })
    .map_err(relogin_cli::subir)?;
    let quantidade = resultado.itens.len();
    let faixa = if quantidade == 0 {
        serde_json::Value::Null
    } else {
        json!({"inicio":1,"fim":quantidade})
    };
    let requisicoes = relogin_cli::somar(resultado.requisicoes, nota.as_ref());
    if json_mode {
        let mut valor = json!({
            "unidade": atual,
            "origem":"web",
            "mes":resultado.mes,
            "total_declarado":quantidade,
            "faixa_observada":faixa,
            "retornos": resultado.itens,
            "requisicoes":requisicoes,
        });
        relogin_cli::anotar(&mut valor, nota.as_ref());
        println!("{valor}");
    } else {
        relogin_cli::avisar(false, nota.as_ref());
        println!(
            "Retornos da unidade {atual} em {}: {} observado(s); {requisicoes} requisicao(oes).",
            resultado.mes, quantidade
        );
        for item in resultado.itens {
            println!(
                "{} | {} | {} | {}",
                item.situacao, item.numero, item.unidade, item.data_programada
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    #[test]
    fn mes_invalido_e_erro_de_uso_do_clap() {
        assert!(
            crate::Cli::try_parse_from(["ies", "unidade", "retornos", "--mes", "2026-13"]).is_err()
        );
    }
    #[test]
    fn forma_publica_da_cli() {
        for args in [
            vec!["ies", "unidade", "retornos"],
            vec!["ies", "unidade", "retornos", "--mes", "2026-10"],
            vec!["ies", "--json", "unidade", "retornos", "--mes", "2026-10"],
        ] {
            assert!(crate::Cli::try_parse_from(args).is_ok());
        }
        assert!(crate::Cli::try_parse_from(["ies", "unidade", "retornos", "--mes"]).is_err());
    }
}
