//! Projeção Web de uma categoria integralmente observada, sem POST.
use crate::paginas::Leitura;
use crate::{arquivo_caixa, configuracao, erro_fonte, paginas, transporte};
use clap::ValueEnum;
use serde_json::{Value, json};
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum CategoriaLista {
    Recebidos,
    Gerados,
}

impl CategoriaLista {
    pub(crate) fn rotulo(self) -> &'static str {
        match self {
            Self::Recebidos => "recebidos",
            Self::Gerados => "gerados",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FalhaCategoria {
    UnidadeNaoObservada,
    PaginasNaoObtidas(CategoriaLista),
}
impl FalhaCategoria {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::UnidadeNaoObservada => "identificacao_da_unidade_ausente_no_controle",
            Self::PaginasNaoObtidas(CategoriaLista::Recebidos) => "paginas_recebidos_nao_obtidas",
            Self::PaginasNaoObtidas(CategoriaLista::Gerados) => "paginas_gerados_nao_obtidas",
        }
    }
    pub(crate) fn mensagem(self) -> &'static str {
        match self {
            Self::UnidadeNaoObservada => "a unidade não está identificada no Controle de Processos",
            Self::PaginasNaoObtidas(_) => {
                "a categoria selecionada exige páginas que não foram obtidas"
            }
        }
    }
}

/// Projeta apenas a categoria com posições inteiramente observadas no GET.
pub(crate) fn selecionar(
    leitura: &Leitura,
    categoria: CategoriaLista,
    origem: &str,
) -> Result<Value, FalhaCategoria> {
    let (tabela, cobertura) = match categoria {
        CategoriaLista::Recebidos => (&leitura.caixa.recebidos, &leitura.recebidos),
        CategoriaLista::Gerados => (&leitura.caixa.gerados, &leitura.gerados),
    };
    if !cobertura.completa(tabela) {
        return Err(FalhaCategoria::PaginasNaoObtidas(categoria));
    }
    Ok(json!({
        "unidade": leitura.unidade,
        "origem": origem,
        "categoria": categoria.rotulo(),
        "total_declarado": tabela.total_declarado,
        "processos": tabela.processos,
    }))
}

/// Uma consulta HTTPS já existente; sem POST nem busca especulativa.
pub(crate) fn listar_categoria(
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&Path>,
    categoria: CategoriaLista,
) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let html = match transporte::read_box_from_file(
        &arquivos.url_controle,
        &arquivos.cookie,
        &arquivos.origem,
    ) {
        Ok(html) => html,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let leitura = match paginas::juntar(&[html], unidade) {
        Ok(leitura) => leitura,
        Err(erro) => {
            return erro_fonte(
                json_mode,
                erro.subcodigo(),
                "Controle de Processos não reconhecido",
            );
        }
    };
    let modelo = match selecionar(&leitura, categoria, "web") {
        Ok(modelo) => modelo,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    if let Some(caminho) = destino {
        return match arquivo_caixa::gravar(caminho, &modelo) {
            Ok(bytes) => {
                if json_mode {
                    println!(
                        "{}",
                        json!({
                            "arquivo":"gravado",
                            "bytes":bytes,
                            "escopo":"categoria_verificada",
                            "categoria":categoria.rotulo()
                        })
                    );
                } else {
                    println!(
                        "IES: categoria verificada gravada em arquivo privado ({bytes} bytes)."
                    );
                }
                ExitCode::SUCCESS
            }
            Err(erro) => erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
        };
    }
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&modelo).unwrap());
    } else {
        let quantidade = modelo["processos"].as_array().map_or(0, Vec::len);
        println!(
            "IES: {}: {quantidade} processo(s) observados e cobertos na categoria; demais categorias não consultadas integralmente.",
            categoria.rotulo()
        );
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paginas;

    fn leitura(recebidos_total: usize) -> Leitura {
        let html = format!(
            "<table id='tblProcessosRecebidos'>\
             <caption>{recebidos_total} registros - 1 a 1):</caption>\
             <tr id='P1'><td><a class='processoVisualizado'>R-0001</a></td></tr></table>\
             <table id='tblProcessosGerados'><caption>1 registros - 1 a 1):</caption>\
             <tr id='P2'><td><a class='processoVisualizado'>G-0002</a></td></tr></table>"
        );
        paginas::juntar(&[html], None).unwrap()
    }

    #[test]
    fn tipo_de_categoria_da_cli_aceita_somente_valores_do_dominio() {
        use clap::Parser;
        assert!(
            crate::Cli::try_parse_from([
                "ies",
                "unidade",
                "listar-processos",
                "--categoria",
                "gerados"
            ])
            .is_ok()
        );
        assert!(
            crate::Cli::try_parse_from([
                "ies",
                "unidade",
                "listar-processos",
                "--categoria",
                "desconhecida"
            ])
            .is_err()
        );
    }

    #[test]
    fn categoria_e_paginas_privadas_nao_podem_ser_combinadas_silenciosamente() {
        use clap::Parser;
        assert!(
            crate::Cli::try_parse_from([
                "ies",
                "unidade",
                "listar-processos",
                "--categoria",
                "gerados",
                "--pagina-arquivo",
                "pagina-2.html"
            ])
            .is_err()
        );
    }

    #[test]
    fn gerados_inteiros_sao_consultaveis_mesmo_se_recebidos_paginam() {
        let modelo = selecionar(&leitura(200), CategoriaLista::Gerados, "web").unwrap();
        assert_eq!(modelo["categoria"], "gerados");
        assert_eq!(modelo["total_declarado"], 1);
        assert_eq!(modelo["processos"][0]["numero"], "G-0002");
        assert!(modelo.get("recebidos").is_none());
        assert!(modelo.get("completa").is_none());
    }

    #[test]
    fn categoria_requerida_ainda_incompleta_falha_sem_publicar_resultado() {
        assert_eq!(
            selecionar(&leitura(200), CategoriaLista::Recebidos, "web"),
            Err(FalhaCategoria::PaginasNaoObtidas(CategoriaLista::Recebidos))
        );
    }

    #[test]
    fn categoria_unica_com_todas_posicoes_preserva_numeros() {
        let modelo = selecionar(&leitura(1), CategoriaLista::Recebidos, "web").unwrap();
        assert_eq!(modelo["processos"][0]["numero"], "R-0001");
        assert_eq!(modelo["total_declarado"], 1);
        assert!(modelo.get("gerados").is_none());
    }
}
