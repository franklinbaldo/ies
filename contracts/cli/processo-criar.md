---
type: CliCommandContract
title: Iniciar processo pelo cadastro Web do SEI
operation: processo.criar
status: proposed
implementation: not-started
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
A resposta sozinha nunca prova a criação.

Falha definitivamente anterior ao envio: saída 1 (ou 2
para argumentos inválidos). Depois de iniciar a escrita,
qualquer resultado que não permita estabelecer a identidade
e o efeito exige `efeito_incerto`, saída 3, sem repetir POST.
O usuário deve conferir a unidade antes de nova tentativa.

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

Esses subcódigos precisam de tipos reais e integração a
`docs/erros-tipados.md` antes de serem publicados na CLI.

## Orçamento e evidência

Argumento rejeitado sem rede: 0. Prévia: consultas necessárias
para abrir lista completa, escolher tipo e conferir o cadastro,
incluindo AJAX somente quando o pedido o exigir; zero POST de
**criação**. Confirmação: mesmas consultas, **um** POST criador
e leitura nova de verificação. O número exato de requisições
por versão ainda precisa de teste replay e medição, não de
estimativa. O POST que apenas escolhe o tipo em SEI 4.1+
não é a escrita criadora.

[Observação](../../docs/como-o-sei-funciona/inicio-do-processo.md)
e [decisão](../../docs/decisoes/processo-criar.md).
