    use super::*;
    const RESULTADO: &str = r#"<html><table class="pesquisaResultado">
      <tr class="pesquisaTituloRegistro"><td class="pesquisaTituloEsquerda">
      <a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7">00001.123456/2026-07</a>
      </td></tr></table></html>"#;

    #[test]
    fn numero_encontrado_na_pesquisa_preserva_zeros_e_nao_link_assinado() {
        let encontrados = ler_resultados(RESULTADO).unwrap();
        assert_eq!(encontrados.len(), 1);
        assert_eq!(encontrados[0].numero.as_str(), "00001.123456/2026-07");
        let saida = serde_json::to_string(&encontrados).unwrap();
        assert!(!saida.contains("id_procedimento"));
        assert!(!saida.contains("controlador.php"));
    }

    #[test]
    fn nao_converte_formulario_vazio_em_resultado_de_busca() {
        assert_eq!(
            ler_resultados("<form id='frmPesquisaProtocolo'></form>"),
            Err(ErroResultadoPesquisa::TabelaDePesquisaAusente)
        );
    }

    #[test]
    fn link_para_outra_acao_nao_vira_processo() {
        let html = RESULTADO.replace("acao=procedimento_trabalhar", "acao=documento_imprimir_web");
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)
        );
    }

    #[test]
    fn numeros_repetidos_da_mesma_resposta_sao_recusados() {
        let html = RESULTADO.replace("</table>", r#"<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8">00001.123456/2026-07</a></td></tr></table>"#);
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::NumeroDePesquisaRepetido)
        );
    }

    #[test]
    fn recusa_duas_acoes_no_mesmo_link_antes_de_projetar_nup() {
        let html = RESULTADO.replace(
            "&amp;id_procedimento=7",
            "&amp;acao=documento_imprimir_web&amp;id_procedimento=7",
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)
        );
    }

    #[test]
    fn duas_tabelas_de_pesquisa_nao_podem_ocultar_resultados() {
        let html = format!(
            "{RESULTADO}<table class='pesquisaResultado'><tr class='pesquisaTituloRegistro'><td><a class='protocoloNormal' href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8'>00002.123456/2026-07</a></td></tr></table>"
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::TabelasDePesquisaRepetidas)
        );
    }

    #[test]
    fn linha_com_dois_links_de_processos_nao_escolhe_o_primeiro_silenciosamente() {
        let html = RESULTADO.replace(
            "</a>",
            r#"</a><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8">00002.123456/2026-07</a>"#,
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinhaDePesquisaComMaisDeUmLinkDoProcesso)
        );
    }

    #[test]
    fn linha_sem_classe_nao_pode_ocultar_processo_da_contagem() {
        // A tabela continua contendo um link SEI de processo legítimo, mas
        // sua linha perdeu a classe de resultado: ignorá-la perde um NUP.
        let html = RESULTADO.replace(
            "</table>",
            r#"<tr><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8">00002.123456/2026-07</a></td></tr></table>"#,
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDeProcessoForaDaLinhaDePesquisa)
        );
    }

    #[test]
    fn link_de_trabalho_sem_classe_css_nao_pode_sumir_do_resultado() {
        // Uma versão do HTML pode alterar a classe do link e da linha.
        // O link ainda é de abrir processo: não retornar somente os
        // resultados cujas classes antigas o parser reconheceu.
        let html = RESULTADO.replace(
            "</table>",
            r#"<tr><td><a href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8">00002.123456/2026-07</a></td></tr></table>"#,
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDeProcessoSemClasseReconhecida)
        );
    }

    #[test]
    fn link_de_outra_acao_sem_classe_nao_cria_processo() {
        let html = RESULTADO.replace(
            "</table>",
            r#"<tr><td><a href="controlador.php?acao=usuario_selecionar">Outra ação</a></td></tr></table>"#,
        );
        assert_eq!(ler_resultados(&html).unwrap().len(), 1);
    }

    #[test]
    fn chave_amp_acao_no_primeiro_parametro_nao_e_acao_valida() {
        let html = RESULTADO.replace(
            "?acao=procedimento_trabalhar",
            "?amp;acao=procedimento_trabalhar",
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaNaoAbreProcesso)
        );
    }

    #[test]
    fn link_sem_identificador_interno_nao_e_um_link_de_processo() {
        let html = RESULTADO.replace("&amp;id_procedimento=7", "");
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaSemIdentificadorDoProcesso)
        );
    }

    #[test]
    fn link_com_identificadores_repetidos_nao_escolhe_destino_arbitrario() {
        let html = RESULTADO.replace(
            "&amp;id_procedimento=7",
            "&amp;id_procedimento=7&amp;id_procedimento=8",
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadoresRepetidos)
        );
    }

    #[test]
    fn link_com_id_interno_nao_decimal_nao_vira_processo_observado() {
        for id in ["abc", "-7", "%37"] {
            let html = RESULTADO.replace(
                "&amp;id_procedimento=7",
                &format!("&amp;id_procedimento={id}"),
            );
            assert_eq!(
                ler_resultados(&html),
                Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadorDoProcessoNaoDecimal),
                "identificador interno {id} deve ser decimal ASCII"
            );
        }
    }

    #[test]
    fn dois_nups_distintos_nao_podem_apontar_para_o_mesmo_processo_interno() {
        let html = RESULTADO.replace(
            "</table>",
            r#"<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7">00002.123456/2026-07</a></td></tr></table>"#,
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::IdentificadorDoProcessoRepetidoNaPesquisa)
        );
    }

    #[test]
    fn id_interno_com_zeros_a_esquerda_nao_pode_representar_dois_processos() {
        let html = RESULTADO.replace(
            "</table>",
            r#"<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal" href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=007">00002.123456/2026-07</a></td></tr></table>"#,
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::IdentificadorDoProcessoRepetidoNaPesquisa),
            "o mesmo ID numérico (7 e 007) não pode identificar NUPs diferentes"
        );
    }

    #[test]
    fn chaves_que_o_php_pode_normalizar_nao_alteram_identidade_silenciosamente() {
        // PHP converte pontos e espacos em '_' e decodifica percent/+
        // antes de escolher o valor do parametro; o parser nao pode
        // aceitar somente a chave literal e ignorar seu alias.
        for extra in [
            "id.procedimento=8",
            "id+procedimento=8",
            "id%5Fprocedimento=8",
            "a%63ao=documento_imprimir_web",
        ] {
            let html = RESULTADO.replace(
                "&amp;id_procedimento=7",
                &format!("&amp;id_procedimento=7&amp;{extra}"),
            );
            assert_eq!(
                ler_resultados(&html),
                Err(ErroResultadoPesquisa::LinkDePesquisaComChaveNaoCanonica),
                "chave alternativa nao pode mudar a acao ou o ID: {extra}"
            );
        }
    }

    #[test]
    fn parametro_acao_sem_igual_nao_pode_sobrescrever_acao_valida() {
        // O parse de query do PHP trata `acao` sem "=" como valor vazio.
        // Um valor vazio posterior sobrepõe a ação reconhecida pelo parser.
        let html = RESULTADO.replace("&amp;id_procedimento=7", "&amp;id_procedimento=7&amp;acao");
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaComAcaoSemIgual)
        );
    }

    #[test]
    fn parametro_id_sem_igual_nao_pode_sobrescrever_identificador_valido() {
        let html = RESULTADO.replace(
            "&amp;id_procedimento=7",
            "&amp;id_procedimento=7&amp;id_procedimento",
        );
        assert_eq!(
            ler_resultados(&html),
            Err(ErroResultadoPesquisa::LinkDePesquisaComIdentificadorSemIgual)
        );
    }

    #[test]
    fn parametros_adicionais_com_nomes_canonicos_preservam_resultado() {
        let html = RESULTADO.replace(
            "&amp;id_procedimento=7",
            "&amp;id_procedimento=7&amp;infra_hash=exemplo&amp;id-contexto=8",
        );
        assert_eq!(ler_resultados(&html).unwrap().len(), 1);
    }

    #[test]
    fn tabela_sem_processos_observados_nao_inventa_identidade() {
        assert_eq!(
            ler_resultados("<table class='pesquisaResultado'></table>"),
            Ok(vec![])
        );
    }
