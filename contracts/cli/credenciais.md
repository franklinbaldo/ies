---
type: CommandContract
title: Senha do login no cofre do sistema operacional
operation: credenciais.cofre
command: ies credenciais salvar|status|excluir --origem URL --usuario LOGIN
effect: read
implementation: partial
---

# Senha do login no cofre do sistema operacional

`ies credenciais` guarda, confere ou apaga a senha do login no cofre do
sistema operacional: Keychain (macOS), Gerenciador de Credenciais
(Windows) ou Secret Service (Linux, pela sessão D-Bus). **Nenhum efeito
no SEI**: nenhuma requisição, nenhuma sessão. O efeito é local, só no
cofre da pessoa.

## Entrada

- `--origem URL`: a URL base HTTPS da instalação, no formato do arquivo
  de origem (`https://host[/base]`, sem consulta, fragmento ou usuário).
  Fora desse formato, saída 2 pelo clap, antes do cofre.
- `--usuario LOGIN`: o login no SEI, aparado; vazio é saída 2 pelo clap.
- A **senha** só entra pelo terminal, **sem eco** (`salvar`). Nunca por
  argumento, variável de ambiente ou entrada padrão redirecionada. É
  gravada byte a byte como digitada (espaços nas bordas incluídos); só
  vazia ou só espaços é recusa.

## Chave

Uma entrada por instalação e usuário: serviço `ies:<host>` (o host da
origem, em minúsculas) e conta igual ao usuário. Instalações no mesmo
host compartilham a entrada. Não há chave legada nem cópia em outro
serviço.

## Subcomandos

- `salvar`: pede a senha e grava (ou substitui) a entrada.
- `status`: lê a entrada. Ausente não é falha.
- `excluir`: apaga a entrada e **relê**; sucesso só quando a releitura
  não acha mais a senha. Entrada que já não existia é `ausente`.

## Saída

```json
{"cofre":"ies:sei.exemplo.gov.br","senha":"gravada"}
```

`senha` é `gravada`, `presente`, `ausente` ou `excluida`. Sem `--json`,
uma linha `IES: senha <estado> no cofre do sistema (ies:<host>).`. A saída
nunca traz o usuário nem a senha.

## Falhas tipadas

Saída 1 com `Error: ErroCredenciais::…` em stderr:

- `SenhaNaoLida(io)`: sem terminal interativo para ler sem eco.
- `SenhaVazia`.
- `Cofre(ErroCofre)`: `CofreIndisponivel { mensagem_do_sistema }` (sem
  Secret Service ou sessão D-Bus, como em servidor, CI ou contêiner;
  plataforma sem suporte), `AcessoNegado { mensagem_do_sistema }` (cofre
  bloqueado ou pedido recusado) e `FalhaDoCofre { mensagem_do_sistema }`.
  Só a entrada inexistente é ausência; qualquer outra falha do cofre é
  erro, nunca `ausente`. A mensagem é a do sistema, sem os bytes da senha.
- `SenhaPresenteAposExcluir`: o cofre aceitou a exclusão e a releitura
  ainda achou a senha.

## Uso pelo login

O arquivo de `IES_CREDENCIAIS_FILE` sem a linha `senha=` faz `ies login`,
o relogin das leituras, a validação de acesso do processo e
`documento assinar --confirmar` lerem a senha desta entrada, pela origem
da sessão e pelo `usuario=` do arquivo
([sessão e login](sessao-login.md#entrada)). Sem cofre utilizável
(servidor, CI), a linha `senha=` no arquivo `0600` continua valendo.
O assistente [`ies wizard`](wizard.md) usa a mesma entrada.
