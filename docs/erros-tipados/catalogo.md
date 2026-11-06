---
type: Guide
title: Catálogo de subcódigos
description: Catálogo literal por contexto, nomes contextuais e históricos e literais passados por helpers
---

# Catálogo de subcódigos

Parte do [guia de erros tipados](../erros-tipados.md), separada por responsabilidade.

## Catálogo literal por contexto

As tabelas complementares enumeram os literais, inclusive os antes
citados só por curinga ou delegação. A verificação procura a união dos
contratos: não exige duplicar cada erro de transporte em cada operação.

- [Sessão, configuração e transporte](../../contracts/cli/subcodigos-sessao.md).
- [Controle, listas e atributos HTML](../../contracts/cli/subcodigos-controle.md).
- [Pesquisa de processos e documentos](../../contracts/cli/subcodigos-pesquisa.md).
- [Árvore e vínculos](../../contracts/cli/subcodigos-arvore.md).
- [Histórico, propriedades e marcadores](../../contracts/cli/subcodigos-processo.md).
- [Documentos, versões e arquivos locais](../../contracts/cli/subcodigos-documento.md).
- [Prévias, escritas e verificações](../../contracts/cli/subcodigos-escrita.md).

São inventários conservadores dos leitores, não prova de alcançabilidade
de cada variante em toda instalação. Códigos aninhados em `falha` ou
`verificacao` conservam a identidade do emissor. O literal isolado não
identifica necessariamente um único contexto.

## Nomes contextuais e históricos

Os subcódigos das quatro condições antes reutilizadas agora identificam o
contexto diretamente, sem depender da etapa e sem aliases (decisão
[#229](https://github.com/franklinbaldo/ies/issues/229)).

| Subcódigo atual | Fato identificado |
| --- | --- |
| `html_da_caixa_invalido` | HTML da caixa não interpretado. |
| `html_do_historico_invalido` | HTML do histórico não interpretado. |
| `numero_da_caixa_invalido` | Número do processo inválido na caixa. |
| `numero_do_historico_invalido` | Número do processo inválido no título do histórico. |
| `pagina_da_caixa_fora_do_intervalo` | Página solicitada fora do intervalo da caixa, resumida ou detalhada. |
| `pagina_da_pesquisa_fora_do_intervalo` | Página solicitada fora do intervalo da pesquisa. |
| `redirecionamento_do_login_sem_localizacao` | Resposta do login sem localização de redirecionamento. |
| `redirecionamento_da_troca_sem_localizacao` | Resposta da troca de unidade sem localização. |

### Subcódigos históricos

Estes nomes foram aposentados diretamente pela
[#229](https://github.com/franklinbaldo/ies/issues/229), sem compatibilidade
transitória e sem emissão nas versões novas.

| Subcódigo histórico | Substituição |
| --- | --- |
| `html_invalido` | `html_da_caixa_invalido` / `html_do_historico_invalido` |
| `numero_invalido` | `numero_da_caixa_invalido` / `numero_do_historico_invalido` |
| `pagina_fora_do_intervalo` | `pagina_da_caixa_fora_do_intervalo` / `pagina_da_pesquisa_fora_do_intervalo` |
| `redirecionamento_sem_localizacao` | `redirecionamento_do_login_sem_localizacao` / `redirecionamento_da_troca_sem_localizacao` |

Outros reusos analisados em #229 mantêm a mesma condição factual,
independentemente do emissor: `unidade_da_tela_diferente_da_solicitada`,
`unidade_exibida_diferente_da_solicitada`,
`controle_web_nao_reconhecido`, `arquivo_privado_vazio` e
`arquivo_privado_link_simbolico`. O contexto indica onde a falha ocorreu,
mas não altera o significado desses fatos. Eles não exigem renomeação.

## Literais passados por helpers da CLI

Estas emissões não estão em métodos `subcodigo`: os helpers recebem o
literal por parâmetro, ou o setup devolve um `Result` com literal. O gate
pode avisar que não as encontrou; isso não autoriza removê-las do contrato.

| Subcódigo | Condição e emissor |
| --- | --- |
| `opcoes_incompativeis_com_busca` | `processo buscar` recebeu opções da leitura de processo. |
| `sem_andamentos_incompativel_com_andamentos_completos` | Leitura de processo recebeu simultaneamente as duas opções incompatíveis. |
| `leitura_diretorio_copias_falhou` | `documento salvar --listar-copias` não conseguiu obter o diretório de trabalho. |
| `escrita_remota_nao_validada` | `documento salvar --confirmar`: confirmação remota ainda bloqueada. |
| `caminho_privado_ausente_ou_nao_absoluto` | Setup: caminho de um dos três arquivos não é absoluto ou não aponta a arquivo. |
| `arquivo_privado_nao_lido` | Setup: metadados de arquivo privado não puderam ser lidos. |
| `permissoes_de_arquivo_privado_inseguras` | Setup no Unix: arquivo privado com permissões para grupo ou outros. |
| `configuracao_deve_ter_caminho_absoluto` | Setup: destino da configuração não é absoluto. |
| `configuracao_nao_serializada` | Setup: falha ao serializar os caminhos em JSON. |
| `configuracao_ja_existe_ou_sem_permissao` | Setup: criação exclusiva do destino recusada, inclusive se já existe. |
| `configuracao_nao_gravada` | Setup: escrita do JSON falhou após abertura do arquivo. |
