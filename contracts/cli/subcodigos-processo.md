---
type: Contract
title: Subcódigos de projeções do processo
description: Inventário literal complementar de falhas por contexto emissor
---

# Subcódigos de projeções do processo

Este complemento enumera as condições de abertura e leituras de histórico, propriedades e marcadores do processo.
As tabelas completam os contratos das operações; não mudam códigos de
saída, orçamento nem regras de recuperação. Delegações preservam o
subcódigo do leitor chamado. O mesmo literal pode aparecer em `subcodigo`
ou em uma falha aninhada; interpretar junto à operação e à etapa.
Ver [regras e contextos compartilhados](../../docs/erros-tipados.md).

Contratos de operação: [Abertura](processo-obter-arvore.md), [andamentos](processo-obter-andamentos.md), [marcadores](processo-obter-marcadores.md) e [propriedades](processo-obter-propriedades.md).

## Falhas tipadas

### historico

Emissor: [src/historico.rs](../../src/historico.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `cabecalhos_html_do_historico_repetidos` | Cabeçalhos HTML do histórico repetidos. |
| `historico_ausente` | Histórico ausente. |
| `historico_invalido` | Histórico inválido. |
| `html_do_historico_invalido` | HTML do histórico inválido. |
| `historico_vazio` | Histórico vazio. |
| `linha_invalida` | Linha inválida. |
| `numero_do_historico_invalido` | Número extraído do título do histórico inválido. |
| `numero_do_historico_diferente_do_solicitado` | Número do histórico diferente do solicitado. |
| `tabelas_do_historico_repetidas` | Tabelas do histórico repetidas. |
| `titulo_ausente` | Título ausente. |
| `titulo_invalido` | Título inválido. |
| `titulos_do_historico_repetidos` | Títulos do histórico repetidos. |

### historico_processo

Emissor: [src/historico_processo.rs](../../src/historico_processo.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `andamentos_alem_do_total_declarado` | Andamentos além do total declarado. |
| `charset_do_historico_nao_suportado` | Charset do histórico não suportado. |
| `endereco_do_historico_de_outro_processo` | Endereço do histórico de outro processo. |
| `endereco_do_historico_nao_reconhecido` | Endereço do histórico não reconhecido. |
| `html_da_arvore_nao_interpretado` | HTML da árvore não interpretado. |
| `legenda_dos_andamentos_ausente` | Legenda dos andamentos ausente. |
| `legenda_dos_andamentos_fora_da_forma_observada` | Legenda dos andamentos fora da forma observada. |
| `link_do_historico_ausente_na_arvore` | Link do histórico ausente na árvore. |
| `link_do_historico_fora_da_forma_observada` | Link do histórico fora da forma observada. |
| `links_do_historico_repetidos_na_arvore` | Links do histórico repetidos na árvore. |
| `regra_de_ver_historico_nao_observada` | Regra de ver histórico não observada. |
| `tipo_do_historico_nao_publicado` | Tipo do histórico não publicado. |
| `visao_do_historico_diferente_da_pedida` | Visão do histórico diferente da pedida. |

### marcadores_processo

Emissor: [src/marcadores_processo.rs](../../src/marcadores_processo.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `cabecalho_dos_marcadores_nao_reconhecido` | Cabeçalho dos marcadores não reconhecido. |
| `endereco_dos_marcadores_de_outro_processo` | Endereço dos marcadores de outro processo. |
| `endereco_dos_marcadores_nao_reconhecido` | Endereço dos marcadores não reconhecido. |
| `html_dos_marcadores_nao_interpretado` | HTML dos marcadores não interpretado. |
| `id_dos_marcadores_diferente_do_aberto` | ID dos marcadores diferente do aberto. |
| `legenda_dos_marcadores_fora_da_forma_observada` | Legenda dos marcadores fora da forma observada. |
| `linha_do_marcador_fora_da_forma_observada` | Linha do marcador fora da forma observada. |
| `links_dos_marcadores_repetidos_na_barra` | Links dos marcadores repetidos na barra. |
| `marcador_repetido_na_tela` | Marcador repetido na tela. |
| `marcadores_alem_do_total_declarado` | Marcadores além do total declarado. |
| `numero_dos_marcadores_diferente_do_solicitado` | Número dos marcadores diferente do solicitado. |
| `tabelas_dos_marcadores_repetidas` | Tabelas dos marcadores repetidas. |
| `tela_dos_marcadores_ausente` | Tela dos marcadores ausente. |

### processo_obter/erro

Emissor: [src/processo_obter/erro.rs](../../src/processo_obter/erro.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `charset_do_controle_nao_suportado` | Charset do controle não suportado. |
| `link_do_processo_no_controle_nao_reconhecido` | Link do processo no controle não reconhecido. |
| `links_do_processo_repetidos_no_controle` | Links do processo repetidos no controle. |
| `pesquisa_rapida_sem_url_de_trabalho` | Pesquisa rápida sem URL de trabalho. |
| `processo_repetido_no_controle` | Processo repetido no controle. |
| `redirecionamento_nao_reconhecido` | Redirecionamento não reconhecido. |
| `resposta_da_entrada_por_id_nao_reconhecida` | Resposta da entrada por ID não reconhecida. |

### propriedades_processo

Emissor: [src/propriedades_processo.rs](../../src/propriedades_processo.rs).

| Subcódigo | Condição neste contexto |
| --- | --- |
| `assunto_fora_da_forma_observada` | Assunto fora da forma observada. |
| `campo_das_propriedades_ausente` | Campo das propriedades ausente. |
| `campo_das_propriedades_repetido` | Campo das propriedades repetido. |
| `data_de_geracao_fora_da_forma_observada` | Data de geração fora da forma observada. |
| `endereco_das_propriedades_de_outro_processo` | Endereço das propriedades de outro processo. |
| `endereco_das_propriedades_nao_reconhecido` | Endereço das propriedades não reconhecido. |
| `formulario_das_propriedades_ausente` | Formulário das propriedades ausente. |
| `formularios_das_propriedades_repetidos` | Formulários das propriedades repetidos. |
| `html_das_propriedades_nao_interpretado` | HTML das propriedades não interpretado. |
| `id_das_propriedades_diferente_do_aberto` | ID das propriedades diferente do aberto. |
| `interessado_fora_da_forma_observada` | Interessado fora da forma observada. |
| `links_das_propriedades_repetidos_na_barra` | Links das propriedades repetidos na barra. |
| `lista_oculta_das_propriedades_nao_observada` | Lista oculta das propriedades não observada. |
| `nivel_de_acesso_fora_da_forma_observada` | Nível de acesso fora da forma observada. |
| `nivel_de_acesso_nao_marcado` | Nível de acesso não marcado. |
| `numero_das_propriedades_diferente_do_solicitado` | Número das propriedades diferente do solicitado. |
| `opcoes_marcadas_repetidas` | Opções marcadas repetidas. |
| `texto_das_propriedades_nao_decodificado` | Texto das propriedades não decodificado. |
| `tipo_do_processo_inconsistente` | Tipo do processo inconsistente. |
