---
type: CommandContract
title: Iniciar processo pelo cadastro Web do SEI
operation: processo.criar
status: proposed
command: ies processo criar --tipo TIPO --especificacao TEXTO [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo criar` — contrato proposto (#245)

```text
ies processo criar --tipo TIPO --especificacao TEXTO
  [--interessado TEXTO ...] [--nivel publico|restrito]
  [--confirmar]
```

A unidade é a unidade ativa da sessão, sujeita a `--unidade`
global se for conferível na tela. O padrão é acesso público.
Sem `--confirmar` é **prévia**, não escrita. A lista publicada
deve conter exatamente uma entrada do tipo pedido, e o cadastro
aberto deve publicar o mesmo ID. Não inferir tipo ausente de
uma lista apenas dos mais usados: expandir a lista completa
pela ação da tela ou recusar a observação incompleta.

## Prévia

Ler a escolha e o cadastro sem enviar o formulário de criação.
Conferir os campos publicados, o assunto sugerido, o nível
habilitado e, no restrito, a hipótese legal oferecida.
Resolver interessados pedidos na própria tela; recusar
ambiguidade. A especificação é o texto fornecido, nunca um
valor construído por script executado. Exibir o que será
enviado e a contagem de requisições reais.

`--nivel restrito` sem hipótese legal selecionável **não**
autoriza fallback silencioso para público. A sintaxe inicial
não expõe seleção de hipótese: quando a tela não tiver uma
escolha determinística segura, recusar antes da escrita.
Se o SEI oferecer mais de uma hipótese possível, é necessário
ampliar o contrato explicitamente antes de permitir a escrita.

## Confirmação

Uma única submissão do cadastro publicado; sem retry nem
redirect automático. A resposta esperada é um 302 para
a árvore com `id_procedimento`, cuja identidade deve ser
confirmada em leitura nova e associada a um NUP não vazio.
A resposta sozinha nunca prova a criação. No `Location`, aceitar
**uma única** ocorrência da ação `procedimento_trabalhar` e **um único**
`id_procedimento` numérico; quaisquer repetições, mesmo com um
parâmetro inválido, são ambiguidade e resultam em
`efeito_incerto_na_criacao` (saída 3), sem seguir a URL assinada.
Na escolha de tipo por link, aplicar a mesma regra a `acao` e
`id_tipo_procedimento` antes de qualquer GET.

Falha definitivamente anterior ao envio: saída 1 (ou 2
para argumentos inválidos). Depois de iniciar a escrita,
qualquer resultado que não permita estabelecer a identidade
e o efeito exige `efeito_incerto`, saída 3, sem repetir POST.
O usuário deve conferir a unidade antes de nova tentativa.

## Subcódigos do leitor de tipos

| Subcódigo | Fato verificável |
| --- | --- |
| `lista_de_tipos_html_invalido` | HTML da lista não pode ser interpretado |
| `tabela_de_tipos_ausente_ou_repetida` | Tabela de tipos ausente, repetida ou não é tabela |
| `tipo_de_processo_nao_oferecido` | Tipo não consta da lista completa |
| `tipo_de_processo_ambiguo` | Mais de uma âncora com nome exato |
| `tipo_de_processo_sem_acao` | Âncora sem ação segura ou desabilitada |
| `lista_de_tipos_incompleta` | Tipo não encontrado na lista curta; não inferir ausência |
| `cadastro_de_processo_html_invalido` | HTML do cadastro ilegível |
| `cadastro_de_processo_ausente_ou_repetido` | Formulário de cadastro ausente ou repetido |
| `tipo_do_cadastro_divergente` | ID do tipo no cadastro diferente do escolhido |
| `especificacao_nao_publicada_no_cadastro` | Campo de especificação ausente ou desabilitado |
| `assunto_sugerido_nao_unico` | Nenhum ou vários assuntos selecionáveis no cadastro |
| `nivel_de_acesso_nao_oferecido` | Rádio de acesso solicitado não está publicado |
| `nivel_de_acesso_desabilitado` | Rádio publicado mas desabilitado |
| `hipotese_legal_nao_identificada` | Hipótese legal única não publicada |
| `tipo_de_processo_invalido` | Tipo CLI vazio ou contém controle |
| `especificacao_de_processo_invalida` | Especificação CLI vazia ou contém controle |
| `interessado_de_processo_invalido` | Interessado CLI vazio ou contém controle |
| `fluxo_de_criacao_ainda_nao_validado` | Execução remota deliberadamente bloqueada até fluxo seguro completo |

## Subcódigos candidatos

Os nomes abaixo são reservados **como proposta**, ainda sem
implementação Rust e sem afirmar aprovação do verificador:

- `tipo_de_processo_nao_oferecido`
- `tipo_de_processo_ambiguo`
- `lista_de_tipos_incompleta`
- `cadastro_de_processo_desconhecido`
- `assunto_sugerido_nao_unico`
- `nivel_de_acesso_nao_oferecido`
- `hipotese_legal_nao_identificada`
- `interessado_de_processo_ambiguo`
- `criacao_de_processo_recusada`
- `efeito_incerto_na_criacao`

Subcódigos já emitidos pelo leitor offline da lista de tipos:
`lista_de_tipos_html_invalido`, `tabela_de_tipos_ausente_ou_repetida`,
`tipo_de_processo_nao_oferecido`, `tipo_de_processo_ambiguo`,
`tipo_de_processo_sem_acao`, `lista_de_tipos_incompleta`.

Esses subcódigos precisam de tipos reais e integração a
`docs/erros-tipados.md` antes de serem publicados na CLI.


## Subcódigos do fluxo Web

| Subcódigo | Recusa |
| --- | --- |
| `cadastro_de_processo_nao_reconhecido` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `cadastro_nao_devolvido_apos_escolha` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `efeito_incerto_na_criacao` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `falha_de_transporte_na_criacao` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `formulario_de_criacao_nao_reconhecido` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `interessado_de_processo_nao_resolvido` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `link_de_iniciar_processo_ausente` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `link_de_iniciar_processo_nao_reconhecido` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `link_de_iniciar_processo_repetido` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `lista_de_tipos_nao_reconhecida` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `menu_de_criacao_nao_reconhecido` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `tipo_de_processo_sem_identificador` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `unidade_da_criacao_divergente` | Etapa observada não satisfeita; não repetir operação automaticamente |


## Subcódigos do interessado

| Subcódigo | Recusa |
| --- | --- |
| `interessado_controles_ausentes` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `interessado_autocompletar_nao_declarado` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `interessado_consulta_nao_derivada` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `interessado_resposta_nao_reconhecida` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `interessado_de_processo_nao_oferecido` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `interessado_de_processo_ambiguo` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |

## Orçamento e evidência

Argumento rejeitado sem rede: 0. Prévia: consultas necessárias
para abrir lista completa, escolher tipo e conferir o cadastro,
incluindo AJAX somente quando o pedido o exigir; zero POST de
**criação**. Confirmação: mesmas consultas, **um** POST criador
e leitura nova de verificação. O número exato de requisições
por versão ainda precisa de teste replay e medição, não de
estimativa. O POST que apenas escolhe o tipo em SEI 4.1+
não é a escrita criadora. Nos testes com transporte injetado,
a prévia 4.0 tem **3 GET**; a confirmação 4.0 tem **6 pedidos**
(4 GET, 1 POST criador e outro GET; ao todo 5 GET e 1 POST);
a confirmação 4.1 tem **7 pedidos** (5 GET, 1 POST de seleção
e 1 POST criador). Não extrapolar esses totais a instalações reais
sem observação e replay. Em URLs ambíguas recusar antes da próxima
requisição; depois de iniciar a escrita, saída 3 sem retry.

[Observação](../../docs/como-o-sei-funciona/inicio-do-processo.md)
e [decisão](../../docs/decisoes/processo-criar.md).
